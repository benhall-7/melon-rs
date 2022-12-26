// use std::collections::HashMap;

/// A kind of event which can be subscribed to from various sources
#[derive(Default)]
pub struct Event<Arg: Clone> {
    subscriptions: Vec<Subscription<Arg>>,
}

// #[derive(Debug)]
pub struct Subscription<Arg: Clone> {
    action: Box<dyn Fn(Arg) + Send + Sync + 'static>,
}

impl<Arg: Clone> Event<Arg> {
    pub fn subscribe(&mut self, func: impl Into<Box<dyn Fn(Arg) + Send + Sync + 'static>>) {
        self.subscriptions.push(Subscription::new(func));
    }

    pub fn call(&self, arg: Arg) {
        self.subscriptions.iter().for_each(|sub| {
            let action = &sub.action;
            action.as_ref()(arg.clone())
        })
    }
}

impl<Arg: Clone> Subscription<Arg> {
    pub fn new(func: impl Into<Box<dyn Fn(Arg) + Send + Sync + 'static>>) -> Self {
        Self {
            action: func.into(),
        }
    }
}
