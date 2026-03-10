use core::panic;
use std::{collections::{HashMap, HashSet, VecDeque}, ops::Deref, sync::{Arc, RwLock}, thread};
use crate::{CPU, Interrupt, Memory, domain::{FFIFuncTable, FFIFunctionInfo, FFIFunctionSignature}, memory::ByteSerialisable};
use libffi::raw::ffi_type;
use log::info;
use crate::cpu::Program;
use crate::domain::generic_ffi_call;

struct FFIResult {
    fut_id: Id,
    value: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FutureState {
    Cancelled,
    Waiting,
    Complete,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CoroutineState {
    Runnable,
    Suspended,
    Running,
    Finished,
    Cancelled,
}

type Id = usize;

pub struct Future {
    id: Id,
    state: FutureState,
    dependants: HashSet<Id>,   // Coroutines awaiting this future. IDK how or when we can have multiple coros awaiting the same future though
    value: Option<Vec<u8>>
}

impl Future {
    pub fn add_dependant(&mut self, coroutine_id: Id) {
        self.dependants.insert(coroutine_id);
    }

    pub fn set_complete(&mut self) {
        self.state = FutureState::Complete;
    }

}

pub struct Coroutine {
    id: Id,
    priority: i32,              // TODO: Use this as weight and make scheduler have a PQ
    state: CoroutineState,
    depends_on: HashMap<Id, usize>,   // Futures awaited by this coro, with the location to write the value to
    dependant: Option<Id>,      // Future whose value is the return value of this coro, if any
    cpu: CPU
}

impl Coroutine {
    pub fn new(id: Id, priority: i32, program: Program) -> Coroutine{
        Coroutine {
            id: id,
            priority: priority,
            state: CoroutineState::Runnable,
            depends_on: HashMap::new(),
            dependant: None,
            cpu: CPU::with_program(0, program)
        }
    }

    pub fn await_future(&mut self, future_id: Id, write_location: usize) {
        self.depends_on.insert(future_id, write_location);
        self.state = CoroutineState::Suspended;
    }

    pub fn memory_dump(&self) -> Memory {
        return self.cpu.memory.clone();
    }
}

pub struct Scheduler {
    coroutines: HashMap<Id, Coroutine>,
    futures: HashMap<Id, Future>,
    ready_queue: VecDeque<Id>,
    _new_spawned_coro_id: Id,
    _new_spawned_future_id: Id,
    running: bool,
    ffi_func_table: Arc<RwLock<FFIFuncTable>>,
    curr_coro_id: usize
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            coroutines: HashMap::new(),
            futures: HashMap::new(),
            ready_queue: VecDeque::new(),
            _new_spawned_coro_id: 0,     // Id that will be assigned to any coro that spawns, NOT the id of the coro currently being run
            _new_spawned_future_id: 0,
            running: false,
            ffi_func_table: Arc::new(RwLock::new(FFIFuncTable::new())),
            curr_coro_id: 0,
        }
    }

    fn get_new_fut_id(&mut self) -> Id {
        self._new_spawned_future_id += 1;
        return self._new_spawned_future_id
    }

    fn get_new_coro_id(&mut self) -> Id {
        self._new_spawned_coro_id += 1;
        return self._new_spawned_coro_id;
    }

    pub fn spawn_coro(&mut self, program: Program, priority: i32, args: & dyn ByteSerialisable) -> Result<Id, String> {
        let id = self.get_new_coro_id();
        
        let fut_id = self.spawn_fut();

        let mut coroutine = Coroutine::new(id, priority, program);
        coroutine.dependant = Some(fut_id);
        
        {
            let memory = coroutine.cpu.get_memory_mut();
            memory.extend_memory_to(args.get_size());
            memory.write(0, args);
        }

        self.coroutines.insert(id, coroutine);
        self.ready_queue.push_back(id);
        
        info!("Spawned new coroutine with id {}, with future {}", id, fut_id);
        return Ok(fut_id);
    }

    pub fn await_future(&mut self, coroutine_id: Id, future_id: Id, write_location: usize) -> Result<(), String> {
        let coroutine = self.coroutines.get_mut(&coroutine_id)
            .ok_or_else(|| format!("Coroutine {} not found", coroutine_id))?;

        // Create new future if it doesn't exist
        if !self.futures.contains_key(&future_id) {
            panic!("Awaited future {}, which does not exist", future_id);
        }
        
        coroutine.await_future(future_id, write_location);

        // Add coroutine as dependant to future
        if let Some(future) = self.futures.get_mut(&future_id) {
            future.add_dependant(coroutine_id);
        }

        info!("Coroutine {} awaiting future at symbol {}", coroutine_id, future_id);
        Ok(())
    }

    pub fn complete_future(&mut self, future_id: Id, value: Result<& dyn ByteSerialisable, String>) -> Result<(), String> {
        let future: &mut Future = self.futures.get_mut(&future_id)
            .ok_or_else(|| format!("Future at symbol {} not found", future_id))?;


        if future.state != FutureState::Waiting{
            panic!("Tried to set value for future {} which has state {}", future_id, "future.state"); // TODO: impl display for fut state
        }

        future.value = Some(value.clone()?.to_bytes());

        let val = *value.as_ref().map_err(|e| e.clone())?;

        // Wake up all dependent coroutines
        future.set_complete();
        for coroutine_id in &future.dependants {
            if let Some(coroutine) = self.coroutines.get_mut(&coroutine_id) {
                coroutine.state = CoroutineState::Runnable;
                self.ready_queue.push_back(*coroutine_id);
                if let Some(write_location) = coroutine.depends_on.get(&future_id) {
                    coroutine.cpu.get_memory_mut().write(*write_location, val);
                }
                coroutine.depends_on.remove(&future_id);
            }
        }

        future.dependants.clear();

        info!("Completed future at symbol {}", future_id);
        Ok(())
    }

    pub fn complete_future_for(&mut self, future_id: Id, coroutine_id: Id) {
        let value = {
            if let Some(fut) = self.futures.get(&future_id) {
                if let Some(value) = &fut.value {
                    value
                } else {
                    panic!("Future complete but no value");
                }
            } else {
                panic!("Supposedly complete future with id {} not found", future_id);
            }
        };

        if let Some(coroutine) = self.coroutines.get_mut(&coroutine_id) {
            coroutine.state = CoroutineState::Runnable;
            self.ready_queue.push_back(coroutine_id);
            if let Some(write_location) = coroutine.depends_on.get(&future_id) {
                coroutine.cpu.get_memory_mut().write(*write_location, value);
            }
            coroutine.depends_on.remove(&future_id);
        }
    }

    // TODO: We should handle future removal in some reasonable way. Right now, futures live forever, but we should clean them up somewhow.
    pub fn delete_future(&mut self, future_id: Id) {
        self.futures.remove(&future_id);
    }

    pub fn get_next_runnable(&mut self) -> Option<Id> {
        while let Some(id) = self.ready_queue.pop_front() {
            if let Some(coroutine) = self.coroutines.get_mut(&id) {
                if coroutine.state == CoroutineState::Runnable {
                    return Some(id);
                }
            }
        }
        return None;
    }

    pub fn yield_coroutine(&mut self, coroutine_id: Id) -> Result<(), String> {
        let coroutine = self.coroutines.get_mut(&coroutine_id)
            .ok_or_else(|| format!("Coroutine {} not found", coroutine_id))?;
        
        coroutine.state = CoroutineState::Runnable;
        self.ready_queue.push_back(coroutine_id);
        
        info!("Yielded coroutine {}", coroutine_id);
        Ok(())
    }

    pub fn finish_coroutine(&mut self, coroutine_id: Id, result: Result<& dyn ByteSerialisable, String>) -> Result<(), String> {
        {        
            let coroutine: &mut Coroutine = self.coroutines.get_mut(&coroutine_id)
                .ok_or_else(|| format!("Coroutine {} not found", coroutine_id))?;
            
            coroutine.state = CoroutineState::Finished;
        }
        
        let coroutine = self.coroutines.get(&coroutine_id).ok_or_else(|| format!("Coroutine {} not found", coroutine_id))?;
        // If this coroutine has a dependant future, complete it
        if let Some(future_sym) = coroutine.dependant {
            self.complete_future(future_sym.clone(), result)?;
        }

        info!("Finished coroutine {}", coroutine_id);
        Ok(())
    }

    pub fn get_coro(&self, coroutine_id: Id) -> &Coroutine {
        if let Some(coro)= self.coroutines.get(&coroutine_id){
            return coro;
        } else {
            panic!("Current coroutine not found");
        }
    }
    
    fn get_curr_coro_mut(&mut self, coroutine_id: Id) -> &mut Coroutine {
        if let Some(coro)= self.coroutines.get_mut(&coroutine_id){
            return coro;
        } else {
            panic!("Current coroutine not found");
        }
    }

    fn spawn_fut(&mut self) -> Id {
        let fut_id = self.get_new_fut_id();

        let fut = Future {
            id: fut_id,
            state: FutureState::Waiting,
            dependants: HashSet::<Id>::new(),
            value: None
        };
        self.futures.insert(fut_id, fut);
        return fut_id
    }

    fn handle_return(&mut self, coroutine_id: Id, ret_val: & dyn ByteSerialisable, ret_val_addr: usize) -> Result<Option<i8>, String> {
        if let Some(fut_id) = self.get_curr_coro_mut(coroutine_id).dependant {
            if let Some(fut) = self.futures.get(&fut_id) {
                if fut.dependants.len() > 0 {
                    self.complete_future(fut_id, Ok(ret_val))?;
                    self.coroutines.remove(&coroutine_id);
                    
                    if let Some(next_coro_id) = self.get_next_runnable(){
                        self.curr_coro_id = next_coro_id;
                    } else {
                        self.running = false;
                    }
                } else {
                    // main method, dont drop coro so we can see the memory and shit
                    let ret_val = {
                        let curr_coro = self.get_curr_coro_mut(coroutine_id);
                        curr_coro.cpu.memory.read_typed::<i8>(ret_val_addr)
                    };  
                    return Ok(Some(ret_val));
                }
            }
        }
        return Ok(None);
    }
    

    pub fn _run(&mut self) -> Result<i8, String>{

        let (tx, rx) = std::sync::mpsc::channel::<FFIResult>();
        
        if let Some(current_coro_id) = self.get_next_runnable() {
            self.curr_coro_id = current_coro_id;
            self.running = true;
            loop {
                if !self.running {
                    // Block until we get an FFI future completion. 
                    rx.recv().ok().map(|FFIResult{fut_id, value}| {
                        let _ = self.complete_future(fut_id, Ok(&value));
                    });
                }
                
                rx.try_recv().ok().map(|FFIResult{fut_id, value}| {
                    let _ = self.complete_future(fut_id, Ok(&value));
                });

                let interrupt = {
                    if let Some(coro)= self.coroutines.get_mut(&self.curr_coro_id){
                        coro.cpu.run()?
                    } else {
                        panic!("Current coroutine not found");
                    }
                };

                match interrupt {
                    Interrupt::Await(fut_id, return_write_addr) => {
                        if let Some(fut) = self.futures.get_mut(&fut_id) {
                            if fut.state == FutureState::Complete {
                                self.complete_future_for(fut_id, self.curr_coro_id);
                            } else {
                                self.await_future(self.curr_coro_id,fut_id, return_write_addr)?;
                                       
                                if let Some(next_coro_id) = self.get_next_runnable(){
                                    self.curr_coro_id = next_coro_id;
                                } else {
                                    self.running = false;
                                }
                            }
                        }
                    },
                    Interrupt::CreateCoroutine(dest, arg_addr, n_arg_bytes, write_coro_fut_id_addr) => {

                        let (program, args) = {
                            let curr_coro = self.get_curr_coro_mut(self.curr_coro_id);
                            let program = curr_coro.cpu.program.fork_to_pc(dest);
                            let args = curr_coro.cpu.memory.read(arg_addr, n_arg_bytes);
                            (program, args)
                        };

                        let coro_fut_id = self.spawn_coro(program, 0, &args)?;
                        
                        let curr_coro = self.get_curr_coro_mut(self.curr_coro_id);
                        curr_coro.cpu.get_memory_mut().write(write_coro_fut_id_addr, &coro_fut_id);
                    }    
                    Interrupt::Ret(ret_val_addr, n_ret_bytes) => {
                        let ret_val = {
                            let curr_coro = self.get_curr_coro_mut(self.curr_coro_id);
                            curr_coro.cpu.memory.read(ret_val_addr, n_ret_bytes)
                        };

                        if let Some(main_ret_value) = self.handle_return(self.curr_coro_id, &ret_val, ret_val_addr)? {
                            return Ok(main_ret_value);
                        }

                    },
                    Interrupt::DeleteFuture(future_id) => {
                        self.delete_future(future_id);
                    },
                    Interrupt::Ok => {},    // we will never actually get this since CPU.run() just continues without returning in this case
                    Interrupt::EOF => {return Ok(0);},
                    Interrupt::LoadSO(domain_id, lib_path) => {
                        unsafe { if let Err(x) = self.ffi_func_table.write().unwrap().add_domain(domain_id, lib_path) {
                            return Err(format!("Error loading SO for domain {}: {}", domain_id, x.deref()));
                        }};
                    },
                    Interrupt::AddFFIFn(domain_id, function_id, function_name, arg_types, ret_type) => {
                        unsafe { if let Err(x) = self.ffi_func_table.write().unwrap().load_function_from_so(domain_id, FFIFunctionInfo::new(function_id, function_name, arg_types, ret_type)) {
                            return Err(format!("Error loading FFI function from domain {}: {}", domain_id, x.deref()));
                        }};
                    },
                    Interrupt::CallFFIFn(domain_id, function_id, arg_addr, n_arg_bytes, ret_addr) => {
                        let n_ret_bytes = {
                            if let Some(fn_n_ret_bytes) = self.ffi_func_table.read().unwrap().get_n_ret_bytes(domain_id, function_id) {
                                fn_n_ret_bytes
                            } else {
                                return Err(format!("FFI function with id {} not found in domain {}", function_id, domain_id));
                            }
                        };

                        let fut_id = self.spawn_fut();
                        self.get_curr_coro_mut(self.curr_coro_id).cpu.memory.write(ret_addr, &fut_id);
                        
                        let args = {
                            let curr_coro = self.get_curr_coro_mut(self.curr_coro_id);
                            let args = curr_coro.cpu.memory.get_slice(arg_addr, n_arg_bytes).to_owned();
                            args
                        };
                        
                        
                        let ffi = Arc::clone(&self.ffi_func_table);
                        let thread_tx = tx.clone();
                        thread::spawn(move || {
                            let mut ret_buf = Vec::<u8>::with_capacity(n_ret_bytes);
                            
                            unsafe { ffi.read().unwrap().call_function(domain_id, function_id, &args, ret_buf.as_mut_ptr()).unwrap() };
                            thread_tx.send(FFIResult { fut_id, value: ret_buf }).unwrap();
                        });

                        
                    }
                };
            };
        } else {
            return Ok(-1);
        }
        
    }

    pub fn run(&mut self, program: Program) -> Result<(), String>{
        self.spawn_coro(program,  0, &Vec::new())?;
        let x = self.ready_queue.front();
        print!("{}", self.ready_queue.len());
        print!("{:?}", self.ready_queue);
        self._run()?;
        return Ok(());
    }

}