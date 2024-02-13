// #![allow(unused_imports)]
// #![allow(dead_code)]
// #![allow(unused_variables)]


use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering};
use std::thread;
use rand::Rng;

fn main() {
    /// Агенты
    trait Agent {
        fn action(&mut self);
    }

    struct Animal {}

    impl Agent for Animal {
        fn action(&mut self) {
            for _ in 0..20_000_000_u64 {
                // Просто гоняем процессорное время.
                let _ = rand::thread_rng().gen_range(0..2);
            }
        }
    }

    /// Точка мира
    struct Mesh<A: Agent> {
        agent_hash: AtomicUsize,
        agent_ptr: AtomicPtr<A>,
        //is_processed: AtomicBool,
    }

    impl<A: Agent> Default for Mesh<A> {
        fn default() -> Self {
            Mesh {
                agent_hash: Default::default(),
                agent_ptr: AtomicPtr::default(),
                //is_processed: Default::default(),
            }
        }
    }

    /// Мир
    pub struct World<A: Agent> {
        height: usize,
        width: usize,
        // Среда
        landscape: Arc<Vec<Vec<Mesh<A>>>>,
        agents: HashMap<usize, A>,

        hash_count: usize,
    }

    impl<A: Agent + 'static> World::<A> {
        pub fn new() -> Self {
            let rows = 16_usize;
            let cols = 32_usize;

            let mut landscape: Vec<Vec<Mesh<A>>> = Vec::with_capacity(rows);

            for _ in 0..rows {
                // Создаем строку.
                let mut row: Vec<Mesh<A>> = Vec::with_capacity(cols);
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
                agents: HashMap::new(),
                hash_count: 0,
            }
        }

        pub fn add(&mut self, mut agent: A, row: usize, col: usize) {
            self.hash_count += 1;

            self.landscape[row][col].agent_hash.store(self.hash_count, Ordering::SeqCst);
            self.landscape[row][col].agent_ptr.store(&mut agent, Ordering::SeqCst);

            self.agents.insert(self.hash_count, agent);
        }

        pub fn simulate(& self) {
            let mut thread_handles = vec![];

            // 16 потоков
            for row in 0..16 {
                let landscape = Arc::clone(&self.landscape);
                let width = self.width.clone();

                thread_handles.push(thread::spawn(move || {
                    println!("Обрабатывается {} строка среды", row);

                    for col in 0..width {
                        let agent_hash = landscape[row][col].agent_hash.load(Ordering::SeqCst);
                        let agent_ptr = landscape[row][col].agent_ptr.load(Ordering::SeqCst);

                        if !agent_ptr.is_null() {
                            println!("Обрабатываем агента с хеш-кодом {}", agent_hash);
                            unsafe {
                                let mut_ref = agent_ptr.as_mut().expect("Обнаружен нулевой указатель на агента");
                                mut_ref.action();
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
