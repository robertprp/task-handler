use std::fmt::{Debug, Display};
use std::ops::{AddAssign};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{sleep, spawn};
use uuid::Uuid;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
enum PriorityLevel {
    High,
    Medium,
    Low,
}

pub trait TaskHandler {
    fn execute(&self) -> i32; // Update to return an integer result
}

pub struct Task {
    pub id: Uuid,
    pub handler: Box<dyn TaskHandler + Send + Sync>,
    pub priority_level: PriorityLevel,
}

pub trait TaskQueue {
    fn push(&mut self, task: Task);
    fn pop(&mut self) -> Option<Task>;
    fn peek(&self) -> Option<&Task>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn handle(&mut self);
}

pub struct PriorityQueue {
    tasks: Vec<Task>,
}

impl TaskQueue for PriorityQueue {
    fn push(&mut self, task: Task) {
        self.tasks.push(task);
        // Sort with high priority tasks first
        self.tasks.sort_by(|a, b| b.priority_level.cmp(&a.priority_level));
    }

    fn pop(&mut self) -> Option<Task> {
        self.tasks.pop()
    }

    fn peek(&self) -> Option<&Task> {
        self.tasks.first()
    }

    fn len(&self) -> usize {
        self.tasks.len()
    }

    fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    fn handle(&mut self) {
        while let Some(task) = self.pop() {
            let handler = task.handler;
            let priority_level = task.priority_level;
            spawn(move || {
                let result = handler.execute();
                println!("Task with priority {:?} executed with result: {}", priority_level, result);
            });
        }
    }
}

pub struct HardProblem<T>
{
    num1: T,
    num2: T,
}

impl<T> HardProblem<T>
where T: AddAssign + Into<i32> + Clone + Display
{
    pub fn new(num1: T, num2: T) -> Self {
        HardProblem { num1, num2 }
    }

    pub fn solve(&self) -> i32 {
        let mut result = self.num1.clone();
        result += self.num2.clone();
        let int_result: i32 = result.into(); // Convert the result to i32
        sleep(std::time::Duration::from_secs(1));
        int_result
    }
}

impl<T> TaskHandler for HardProblem<T>
where T: AddAssign + Into<i32> + Clone + Display {
    fn execute(&self) -> i32 {
        self.solve()
    }
}

fn main() {
    let (sender, receiver) = mpsc::channel::<Task>();
    let queue = Arc::new(Mutex::new(PriorityQueue { tasks: Vec::new() }));

    // Create initial tasks
    let task1 = Task {
        id: Uuid::new_v4(),
        handler: Box::new(HardProblem::new(3, 2)),
        priority_level: PriorityLevel::High,
    };

    let task2 = Task {
        id: Uuid::new_v4(),
        handler: Box::new(HardProblem::new(5, 7)),
        priority_level: PriorityLevel::Low,
    };

    // Add tasks to the queue
    {
        let mut queue = queue.lock().unwrap();
        queue.push(task1);
        queue.push(task2);
    }

    // Sender thread
    let sender_thread = spawn(move || {
        for i in 0..10 {
            let task = Task {
                id: Uuid::new_v4(),
                handler: Box::new(HardProblem::new(i as i32, (i + 1) as i32)),
                priority_level: if i % 2 == 0 {
                    PriorityLevel::High
                } else {
                    PriorityLevel::Low
                },
            };
            sender.send(task).unwrap();
            sleep(std::time::Duration::from_secs(1));
        }
    });

    // Task handler thread
    let queue_clone = Arc::clone(&queue);
    let task_handler_thread = spawn(move || {
        let mut queue = queue_clone.lock().unwrap();
        queue.handle();
    });

    // Main receiver loop
    let receiver_thread = spawn(move || {
        while let Ok(task) = receiver.recv() {
            let task_result = task.handler.execute();
            println!("Received task with ID: {} produced result: {}", task.id, task_result);

            println!("Adding new task based on result: {}", task_result);
            // Create a new task with updated values based on result
            let medium_priority = Task {
                id: Uuid::new_v4(),
                handler: Box::new(HardProblem::new(task_result, task_result + 1)),
                priority_level: PriorityLevel::Medium,
            };
            
            let high_priority = Task {
                id: Uuid::new_v4(),
                handler: Box::new(HardProblem::new(task_result, task_result + 1)),
                priority_level: PriorityLevel::Low,
            };

            let mut queue = queue.lock().unwrap();
            queue.push(medium_priority);
            queue.push(high_priority);
        }
    });

    // Wait for threads to complete
    sender_thread.join().unwrap();
    task_handler_thread.join().unwrap();
    receiver_thread.join().unwrap();

    println!("All threads completed.");
}
