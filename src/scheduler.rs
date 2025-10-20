use std::collections::{HashMap, VecDeque};
use crate::memory::{Memory, Symbol, Value, Data};
use log::info;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CoroutineId(u64);

pub struct Future {
    value: Option<Result<Value, String>>,
    state: FutureState,
    owner: CoroutineId,
    dependants: VecDeque<CoroutineId>,
}

pub struct Coroutine {
    id: CoroutineId,
    priority: i32,
    state: CoroutineState,
    depends_on: Option<Symbol>,
    depends_on_sym: Symbol,
    dependant: Option<Symbol>,
    memory_state: Memory,
    pc: usize,
}

pub struct Scheduler {
    coroutines: HashMap<CoroutineId, Coroutine>,
    futures: HashMap<Symbol, Future>,
    ready_queue: VecDeque<CoroutineId>,
    next_id: u64,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            coroutines: HashMap::new(),
            futures: HashMap::new(),
            ready_queue: VecDeque::new(),
            next_id: 0,
        }
    }

    pub fn spawn(&mut self, entry_point: Symbol, priority: i32) -> Result<CoroutineId, String> {
        let id = CoroutineId(self.next_id);
        self.next_id += 1;

        let coroutine = Coroutine {
            id,
            priority,
            state: CoroutineState::Runnable,
            depends_on: None,
            depends_on_sym: Symbol("".to_string()), // Placeholder
            dependant: None,
            memory_state: Memory::new(),
            pc: 0,
        };

        self.coroutines.insert(id, coroutine);
        self.ready_queue.push_back(id);
        
        info!("Spawned new coroutine with id {}", id.0);
        Ok(id)
    }

    pub fn await_future(&mut self, coroutine_id: CoroutineId, future_sym: Symbol, result_sym: Symbol) -> Result<(), String> {
        let coroutine = self.coroutines.get_mut(&coroutine_id)
            .ok_or_else(|| format!("Coroutine {} not found", coroutine_id.0))?;

        // Create new future if it doesn't exist
        if !self.futures.contains_key(&future_sym) {
            self.futures.insert(future_sym.clone(), Future {
                value: None,
                state: FutureState::Waiting,
                owner: coroutine_id,
                dependants: VecDeque::new(),
            });
        }

        // Update coroutine state
        coroutine.state = CoroutineState::Suspended;
        coroutine.depends_on = Some(future_sym.clone());
        coroutine.depends_on_sym = result_sym;

        // Add coroutine as dependant to future
        if let Some(future) = self.futures.get_mut(&future_sym) {
            future.add_dependant(coroutine_id);
        }

        info!("Coroutine {} awaiting future at symbol {}", coroutine_id.0, future_sym.0);
        Ok(())
    }

    pub fn complete_future(&mut self, future_sym: Symbol, value: Result<Value, String>) -> Result<(), String> {
        let future = self.futures.get_mut(&future_sym)
            .ok_or_else(|| format!("Future at symbol {} not found", future_sym.0))?;

        // Wake up all dependent coroutines
        let dependants = future.set_value(value);
        for coroutine_id in dependants {
            if let Some(coroutine) = self.coroutines.get_mut(&coroutine_id) {
                coroutine.state = CoroutineState::Runnable;
                self.ready_queue.push_back(coroutine_id);
            }
        }

        info!("Completed future at symbol {}", future_sym.0);
        Ok(())
    }

    pub fn get_next_runnable(&mut self) -> Option<&mut Coroutine> {
        while let Some(id) = self.ready_queue.pop_front() {
            if let Some(coroutine) = self.coroutines.get_mut(&id) {
                if coroutine.state == CoroutineState::Runnable {
                    return Some(coroutine);
                }
            }
        }
        None
    }

    pub fn yield_coroutine(&mut self, coroutine_id: CoroutineId) -> Result<(), String> {
        let coroutine = self.coroutines.get_mut(&coroutine_id)
            .ok_or_else(|| format!("Coroutine {} not found", coroutine_id.0))?;
        
        coroutine.state = CoroutineState::Runnable;
        self.ready_queue.push_back(coroutine_id);
        
        info!("Yielded coroutine {}", coroutine_id.0);
        Ok(())
    }

    pub fn finish_coroutine(&mut self, coroutine_id: CoroutineId, result: Result<Value, String>) -> Result<(), String> {
        let coroutine = self.coroutines.get_mut(&coroutine_id)
            .ok_or_else(|| format!("Coroutine {} not found", coroutine_id.0))?;
        
        coroutine.state = CoroutineState::Finished;
        
        // If this coroutine has a dependant future, complete it
        if let Some(future_sym) = &coroutine.dependant {
            self.complete_future(future_sym.clone(), result)?;
        }

        info!("Finished coroutine {}", coroutine_id.0);
        Ok(())
    }
}