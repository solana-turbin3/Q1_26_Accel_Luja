use std::{collections::VecDeque, error::Error};

use borsh::{BorshDeserialize, BorshSerialize};

use crate::todo::Todo;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Queue<T> {
    pub items: VecDeque<T>,
}
impl<T> Queue<T> {
    pub fn new() -> Self {
        Queue {
            items: VecDeque::new(),
        }
    }
    pub fn enqueue(&mut self, item: T) {
        self.items.push_back(item)
    }
    pub fn dequeue(&mut self) -> Option<T> {
        self.items.pop_front()
    }
    pub fn peek(&self) -> Option<&T> {
        self.items.front()
    }
    pub fn len(&self) -> usize {
        self.items.len()
    }
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl<T> Default for Queue<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub fn save(queue: &Queue<Todo>) -> Result<(), Box<dyn Error>> {
    let items: Vec<Todo> = queue.items.iter().cloned().collect();
    std::fs::write("todo.bin", borsh::to_vec(&items)?).map_err(|e| e.into())
}

pub fn open() -> Result<Queue<Todo>, Box<dyn Error>> {
    let items: Vec<u8> = std::fs::read("todo.bin").unwrap_or_default();
    if items.is_empty() {
        return Ok(Queue::new());
    }
    let todos: Vec<Todo> = borsh::from_slice(&items).unwrap();
    Ok(Queue {
        items: VecDeque::from(todos),
    })
}
pub fn update(queue: &Queue<Todo>) -> Result<(), Box<dyn Error>> {
    let items: Vec<Todo> = queue.items.iter().cloned().collect();
    std::fs::write("todo.bin", borsh::to_vec(&items)?).map_err(|e| e.into())
}
