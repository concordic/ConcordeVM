use core::panic;
use std::{collections::{HashMap, HashSet, VecDeque}, future, hash::Hash};
use crate::{CPU, Interrupt, Memory, memory::ByteSerialisable};
use log::info;

use crate::cpu::Program;


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
    depends_on: HashMap<Id, usize>,   // Futures awaited by this coro
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

    // TODO: double check all spawn methods. The stuff with dependants is messed up, a dependant should be something that awaits a future, but in some places is what the future depends on
    pub fn spawn_coro(&mut self, program: Program, priority: i32, args: & dyn ByteSerialisable) -> Result<Id, String> {
        let id = self.get_new_coro_id();
        
        let fut_id = self.spawn_fut(Some(id));

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


        if future.state == FutureState::Waiting{
            future.value = Some(value.clone()?.to_bytes());
        } else {
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

    fn spawn_fut(&mut self, dep: Option<Id>) -> Id {
        let fut_id = self.get_new_fut_id();

        let mut dependants = HashSet::<Id>::new();
        
        if let Some(dep_id) = dep {
            dependants.insert(dep_id);
        }

        let fut = Future {
            id: fut_id,
            state: FutureState::Waiting,
            dependants,
            value: None
        };
        self.futures.insert(fut_id, fut);
        return fut_id
    }

    pub fn _run(&mut self) -> Result<i8, String>{
        
        if let Some(mut curr_coro_id) = self.get_next_runnable() {
            self.running = true;
            loop {
                 // TODO (SUPER IMPORTANT): DO AN EPOLL OR SOMETHING EQUIV HERE TO RESUME ON AN IO FUTURE COMPLETE, AND TO SET COMPLETED IO FUTURES IN GENERAL
                if !self.running {
                    panic!("As of now, the scheduler cannot exit a state where there is no coro to be run. This needs to be fixed by polling to wait for IO future completion");
                }

                let interrupt = {
                    if let Some(coro)= self.coroutines.get_mut(&curr_coro_id){
                        coro.cpu.run()?
                    } else {
                        panic!("Current coroutine not found");
                    }
                };

                match interrupt {
                    Interrupt::Await(fut_id, return_write_addr) => {
                        self.await_future(curr_coro_id,fut_id, return_write_addr)?;
                        if let Some(fut) = self.futures.get_mut(&fut_id) {
                            if fut.state == FutureState::Complete {
                                self.complete_future_for(fut_id, curr_coro_id);
                            }
                        }
                        
                        if let Some(next_coro_id) = self.get_next_runnable(){
                            self.get_curr_coro_mut(curr_coro_id).cpu.program.pc += 1;
                            curr_coro_id = next_coro_id;
                        } else {
                            self.running = false;
                        }
                    },
                    Interrupt::CreateCoroutine(dest, arg_addr, n_arg_bytes, write_coro_fut_id_addr) => {

                        let (program, args) = {
                            let curr_coro = self.get_curr_coro_mut(curr_coro_id);
                            let program = curr_coro.cpu.program.fork_to_pc(dest);
                            let args = curr_coro.cpu.memory.read(arg_addr, n_arg_bytes);
                            (program, args)
                        };

                        let coro_fut_id = self.spawn_coro(program, 0, &args)?;
                        
                        let curr_coro = self.get_curr_coro_mut(curr_coro_id);
                        curr_coro.cpu.get_memory_mut().write(write_coro_fut_id_addr, &coro_fut_id);
                        curr_coro.cpu.program.pc += 1;
                    }    
                    Interrupt::Ret(ret_val_addr, n_ret_bytes) => {
                        let ret_val = {
                            let curr_coro = self.get_curr_coro_mut(curr_coro_id);
                            curr_coro.cpu.memory.read(ret_val_addr, n_ret_bytes)
                        };
                        if let Some(fut_id) = self.get_curr_coro_mut(curr_coro_id).dependant {
                            if let Some(fut) = self.futures.get(&fut_id) {
                                if fut.dependants.len() > 0 {
                                    self.complete_future(fut_id, Ok(&ret_val))?;
                                    self.coroutines.remove(&curr_coro_id);
                                } else {
                                    // main method, dont drop coro so we can see the memory and shit
                                    let ret_val = {
                                        let curr_coro = self.get_curr_coro_mut(curr_coro_id);
                                        curr_coro.cpu.memory.read_typed::<i8>(ret_val_addr)
                                    };  
                                    return Ok(ret_val);
                                }
                            }
                        }
                    },
                    Interrupt::DeleteFuture(future_id) => {
                        self.delete_future(future_id);
                    },
                    Interrupt::Ok => {},    // we will never actually get this since CPU.run() just continues without returning in this case
                    Interrupt::EOF => {return Ok(0);}
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