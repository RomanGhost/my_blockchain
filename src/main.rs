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
        self.priority.cmp(&other.priority) // Сортировка по возрастанию
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn main() {
    // Создаем защищенную приоритетную очередь, доступную для нескольких потоков
    let queue = Arc::new(Mutex::new(BinaryHeap::new()));

    // Запускаем несколько производителей
    let mut handles = vec![];
    for i in 0..5 {
        let queue_clone = Arc::clone(&queue);
        let handle = thread::spawn(move || {
            for j in 0..10 {
                // Создаем задачу с случайным приоритетом
                let task = Task {
                    priority: rand::random::<u32>() % 100,
                    id: i * 10 + j,
                };
                println!("Производитель {}: добавляет задачу {:?}", i, task);

                // Добавляем задачу в очередь
                queue_clone.lock().unwrap().push(task);

                // Имитация задержки
                thread::sleep(Duration::from_millis(100));
            }
        });
        handles.push(handle);
    }

    // Запускаем потребителя
    let consumer_handle = {
        let queue_clone = Arc::clone(&queue);
        thread::spawn(move || {
            loop {
                let task = {
                    // Блокируем очередь и извлекаем задачу с наивысшим приоритетом
                    let mut queue = queue_clone.lock().unwrap();
                    queue.pop()
                };

                match task {
                    Some(t) => {
                        println!("Потребитель: получил задачу {:?}", t);
                        // Имитация обработки задачи
                        thread::sleep(Duration::from_millis(400));
                    }
                    None => {
                        // Если очередь пуста, даем небольшой таймаут перед следующей попыткой
                        thread::sleep(Duration::from_millis(100));
                        continue;
                    }
                }
            }
        })
    };

    // Ожидаем завершения всех потоков производителей
    for handle in handles {
        handle.join().unwrap();
    }

    // Потребитель продолжает свою работу; при необходимости можно добавить механизм завершения
    consumer_handle.join().unwrap();
}