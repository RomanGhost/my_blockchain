use std::collections::BinaryHeap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Eq, PartialEq, Debug)]
struct Task {
    priority: u32,
    id: u32,
}

// Реализуем Ord для сортировки по приоритету
impl Ord for Task {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.priority.cmp(&self.priority) // Сортировка по убыванию
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn main() {
    let queue = Arc::new(Mutex::new(BinaryHeap::new()));

    // Запускаем несколько производителей
    let mut handles = vec![];
    for i in 0..5 {
        let queue_clone = Arc::clone(&queue);
        let handle = thread::spawn(move || {
            for j in 0..10 {
                let task = Task {
                    priority: rand::random::<u32>() % 100, // Случайный приоритет
                    id: i * 10 + j,
                };
                println!("Производитель {}: добавляет задачу {:?}", i, task);
                queue_clone.lock().unwrap().push(task);
                thread::sleep(Duration::from_millis(100)); // Имитация задержки
            }
        });
        handles.push(handle);
    }

    // Запускаем потребителя
    let consumer_handle = {
        let queue_clone = Arc::clone(&queue);
        thread::spawn(move || {
            for _ in 0..50 { // Ожидаем 50 задач
                let task = {
                    let mut queue = queue_clone.lock().unwrap();
                    queue.pop() // Извлекаем задачу с наивысшим приоритетом
                };
                match task {
                    Some(t) => println!("Потребитель: получил задачу {:?}", t),
                    None => break, // Если очередь пуста, завершаем
                }
                thread::sleep(Duration::from_millis(150)); // Имитация обработки
            }
        })
    };

    // Ожидаем завершения всех потоков
    for handle in handles {
        handle.join().unwrap();
    }

    // Завершаем поток потребителя
    consumer_handle.join().unwrap();
}
