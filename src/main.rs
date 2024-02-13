// #![allow(unused_imports)]
// #![allow(dead_code)]
// #![allow(unused_variables)]


use std::collections::HashMap;
use std::sync::{Arc, LockResult, RwLock};
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering};
use std::thread;
use rand::Rng;

fn main() {
    /// Агенты
    trait Agent {
        fn action(& self);
    }

    struct Animal {}

    impl Agent for Animal {
        fn action(& self) {
            for _ in 0..20_000_000_u64 {
                // Просто гоняем процессорное время.
                let _ = rand::thread_rng().gen_range(0..2);
            }
        }
    }

    /// Точка мира
    struct Mesh {
        agent_hash: AtomicUsize,
        //is_processed: AtomicBool,
    }

    impl Default for Mesh {
        fn default() -> Self {
            Mesh {
                agent_hash: Default::default(),
                //is_processed: Default::default(),
            }
        }
    }

    /// Мир
    pub struct World<A: Agent> {
        height: usize,
        width: usize,
        // Среда
        landscape: Arc<Vec<Vec<Mesh>>>,
        agents: Arc<RwLock<HashMap<usize, RwLock<A>>>>,

        hash_count: usize,
    }

    impl<A: Agent + 'static + std::marker::Send + std::marker::Sync> World::<A> {
        pub fn new() -> Self {
            let rows = 16_usize;
            let cols = 32_usize;

            let mut landscape: Vec<Vec<Mesh>> = Vec::with_capacity(rows);

            for _ in 0..rows {
                // Создаем строку.
                let mut row: Vec<Mesh> = Vec::with_capacity(cols);
                // Проходимся по строке и заполняем ее значениями по умолчанию.
                for _ in 0..cols {
                    row.push(Default::default());
                }
                // Помещаем заполненную строку в контейнер строк.
                landscape.push(row);
            }

            World {
                height: rows,
                width: cols,
                landscape: Arc::new(landscape),
                agents: Arc::new(RwLock::new(HashMap::new())),
                hash_count: 0,
            }
        }

        pub fn add(&mut self, agent: A, row: usize, col: usize) {
            self.hash_count += 1;
            self.landscape[row][col].agent_hash.store(self.hash_count, Ordering::SeqCst);

            let mut guard = self.agents.write().unwrap();
            guard.insert(self.hash_count, RwLock::new(agent));
        }

        pub fn simulate(& self) {
            let mut thread_handles = vec![];

            // 16 потоков
            for row in 0..1 {
                let landscape = Arc::clone(&self.landscape);
                let width = self.width.clone();
                let agents = self.agents.clone();

                thread_handles.push(thread::spawn(move || {
                    println!("Обрабатывается {} строка среды", row);

                    for col in 0..width {
                        let agent_hash = landscape[row][col].agent_hash.load(Ordering::SeqCst);
                        let agents_guard = agents.read().unwrap();
                        let agent = agents_guard.get(&agent_hash);

                        match agent {
                            None => {}
                            Some(a) => {
                                let guard = a.read().unwrap();
                                guard.action();
                            }
                        }
                    }
                }));
            }

            for handle in thread_handles {
                handle.join().unwrap();
            }
        }
    }

    let mut world = World::<Animal>::new();

    let animal = Animal{};
    world.add(animal, 0, 7);

    let animal = Animal{};
    world.add(animal, 1, 9);


    use chrono::Utc;
    use round::round;

    let start = Utc::now().timestamp() as f64;

    world.simulate();

    let end = Utc::now().timestamp() as f64;

    println!("Программа проработала {} минут(ы)", round((end - start)/60.0, 2));
}
