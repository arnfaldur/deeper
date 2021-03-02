use std::time::{Duration, Instant};

type Generation = u32;

pub struct DebugTimer {
    current_gen: Generation,
    stack: Vec<(String, Instant)>,
    popped: Vec<(String, Duration, Generation)>,
}

impl DebugTimer {
    pub fn new() -> Self {
        Self {
            current_gen: 0,
            stack: vec![],
            popped: vec![],
        }
    }

    pub fn push(&mut self, label: &str) {
        self.current_gen += 1;
        self.stack.push((label.to_string(), Instant::now()));
    }

    pub fn pop(&mut self) {
        if let Some((label, instant)) = self.stack.pop() {
            self.current_gen -= 1;
            self.popped
                .push((label, instant.elapsed(), self.current_gen));
        }
    }

    pub fn finish(mut self) -> DebugTimerInfo {
        while !self.stack.is_empty() {
            self.pop();
        }

        DebugTimerInfo::new(self.popped)
    }
}

pub struct TimerInfo {
    pub label: String,
    pub duration: Duration,
    pub children: Vec<TimerInfo>,
}

impl TimerInfo {
    fn new(input: &[(String, Duration, Generation)]) -> Self {
        let (label, duration, generation) = input.last().unwrap();

        let mut children = vec![];

        let input = &input[..input.len() - 1];

        for slice in input.group_by(|(_, _, a), _| *a != *generation + 1) {
            children.push(TimerInfo::new(slice));
        }

        Self {
            children,
            duration: *duration,
            label: label.to_string(),
        }
    }

    #[allow(unused)]
    fn print(&self, generation: usize) {
        let prefix = "| ".repeat(generation);

        println!("* {}, {:?}", self.label, self.duration);

        for child in &self.children {
            println!("| {}", prefix);
            print!("{}+-", prefix);
            child.print(generation + 1);
        }
    }
}

pub struct DebugTimerInfo {
    pub roots: Vec<TimerInfo>,
}

impl DebugTimerInfo {
    fn new(input_vec: Vec<(String, Duration, Generation)>) -> Self {
        println!("{:?}", input_vec);

        let groups = input_vec.group_by(|(_, _, a), _| *a != 0);

        let mut roots = vec![];

        for slice in groups {
            roots.push(TimerInfo::new(slice));
        }

        DebugTimerInfo { roots }
    }

    #[allow(unused)]
    pub fn print(&self) {
        for root in &self.roots {
            root.print(0);
            println!();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut timer = DebugTimer::new();

        assert_eq!(timer.current_gen, 0);
        assert!(timer.stack.is_empty());
        assert!(timer.popped.is_empty());

        timer.push("A");
        assert_eq!(timer.current_gen, 1);
        timer.push("a1");
        assert_eq!(timer.current_gen, 2);

        assert_eq!(timer.stack.len(), 2);
        assert!(timer.popped.is_empty());

        timer.push("a1.b");
        assert_eq!(timer.current_gen, 3);
        timer.pop();
        assert_eq!(timer.current_gen, 2);

        assert!(!timer.popped.is_empty());

        timer.pop();
        assert_eq!(timer.current_gen, 1);

        timer.push("a2");
        assert_eq!(timer.current_gen, 2);
        timer.pop();
        assert_eq!(timer.current_gen, 1);

        timer.pop();
        assert_eq!(timer.current_gen, 0);

        timer.push("B");
        timer.pop();

        assert_eq!(timer.current_gen, 0);

        timer.push("C");
        timer.push("c1");
        timer.pop();
        timer.push("c2");
        timer.pop();

        let info = timer.finish();

        info.print();
    }
}
