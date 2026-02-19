use crate::{
    queue::{Queue, open, save},
    todo::Todo,
};
use std::time::SystemTime;
mod queue;
mod todo;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut queue = open().unwrap();
    match args.get(1).map(|s| s.as_str()) {
        Some("add") => {
            let todo = Todo {
                id: (queue.len() + 1) as u64,
                created_at: SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                description: args.get(2).expect("description").clone(),
            };
            add_task(&mut queue, todo);
            save(&queue).unwrap();
        }
        Some("next") => match queue.peek() {
            Some(todo) => println!(
                "Next todo: id: {},description: {}",
                todo.id, todo.description
            ),
            None => println!("No next"),
        },
        Some("done") => {
            complete_task(&mut queue);
            save(&queue).unwrap();
        }
        Some("list") => {
            list_all_tasks(&queue);
        }
        _ => println!("use add | next | complete task | List"),
    }

    pub fn add_task(queue: &mut Queue<Todo>, todo: Todo) {
        queue.enqueue(todo);
    }
    pub fn list_all_tasks(queue: &Queue<Todo>) {
        for item in queue.items.iter() {
            println!("{:?}", item)
        }
    }
    pub fn complete_task(queue: &mut Queue<Todo>) -> Option<Todo> {
        queue.dequeue()
    }
}
