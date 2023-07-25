/**
Five philosophers dine together at the same table. Each philosopher has their own place at the table. There is a fork between each plate. The dish served is a kind of spaghetti which has to be eaten with two forks. Each philosopher can only alternately think and eat. Moreover, a philosopher can only eat their spaghetti when they have both a left and right fork. Thus two forks will only be available when their two nearest neighbors are thinking, not eating. After an individual philosopher finishes eating, they will put down both forks.
 */
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

type ThreadSafeFork = Arc<Mutex<Fork>>;

struct Fork;

struct Philosopher<'a> {
    name: &'a str,
    left_fork: ThreadSafeFork,
    right_fork: ThreadSafeFork,
    thoughts: mpsc::Sender<String>,
}

impl<'a> Philosopher<'a> {
    fn think(&self) {
        self.thoughts
            .send(format!("Eureka! {} has a new idea!", &self.name))
            .unwrap();
    }

    fn eat(&self) {
        // Pick up forks...
        let left_fork = self.left_fork.try_lock();
        let right_fork = self.right_fork.try_lock();

        if let (Ok(_), Ok(_)) = (left_fork, right_fork) {
            // Only proceed with eating if we were able to acquire both fork locks
            println!("{} is eating...", &self.name);
            thread::sleep(Duration::from_millis(10));

            // Mutex locks on the forks will be released after this closure,
            // because that's how long the `_` placeholder vars live that hold a reference to these locks.
            return;
        }

        // Both forks aren't yet available, need to wait.
        // An alternative to waiting might be to break the fork symmetry such that one of the
        // philosophers locks the right fork before the left, thus preventing deadlocks.
        // I find that alternative solution less elegant though.
        thread::sleep(Duration::from_millis(5));
        println!("{} is waiting for forks before eating...", &self.name);
        self.eat();
    }
}

static PHILOSOPHERS: &[&str] = &["Socrates", "Plato", "Aristotle", "Thales", "Pythagoras"];

fn main() {
    // Create forks
    let forks: Vec<ThreadSafeFork> = (0..PHILOSOPHERS.len())
        .map(|_| Arc::new(Mutex::new(Fork)))
        .collect();

    // Create philosophers
    let (thoughts_tx, thoughts_rx) = mpsc::channel();

    // Make each of them think and eat 100 times
    for (idx, philosopher_name) in PHILOSOPHERS.iter().enumerate() {
        let philosopher_thoughts_tx = thoughts_tx.clone();

        let left_fork = forks
            .get((idx + forks.len() - 1) % forks.len())
            .expect("Left fork should exist")
            .clone();
        let right_fork = forks
            .get((idx + forks.len() + 1) % forks.len())
            .expect("Right fork should exist")
            .clone();

        thread::spawn(move || {
            let philosopher = Philosopher {
                name: philosopher_name,
                left_fork,
                right_fork,
                thoughts: philosopher_thoughts_tx,
            };
            for _ in 0..100 {
                philosopher.eat();
                philosopher.think();
            }
        });
    }

    // Drop the main channel transmitter, so that the channel closes once all child threads finish
    drop(thoughts_tx);

    // Output their thoughts
    for thought in thoughts_rx {
        println!("{thought}");
    }

    println!("done!");
}
