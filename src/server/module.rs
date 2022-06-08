use std::collections::HashMap;
type Handler<P, R> = fn(P) -> R;
pub trait Router<P, R> {
    fn handle<S: Into<String>>(&mut self, submod: S, handler: Option<Handler<P, R>>) -> &Self;
}

#[derive(Debug)]
pub struct ServiceModule<P, R> {
    handle: Option<Handler<P, R>>,
    subs: Option<HashMap<String, Self>>,
}

impl<P, R> ServiceModule<P, R> {
    pub fn new(handler: Option<Handler<P, R>>) -> Self {
        Self {
            handle: handler,
            subs: None,
        }
    }

    pub fn form_keys<O: AsRef<str>>(&self, root: O) -> Option<Vec<String>> {
        if self.subs.is_none() {
            return Some(vec![root.as_ref().to_owned()]);
        }
        let mut keys = vec![];

        for (name, module) in self.subs.as_ref().unwrap().iter() {
            if let Some(mut k) = module.form_keys(name.to_owned()) {
                k = k
                    .iter()
                    .map(|el| root.as_ref().to_owned() + "." + el)
                    .collect();
                keys.append(&mut k);
            }
        }

        Some(keys)
    }

    pub fn get_handler<T: Into<String> + PartialEq>(
        &self,
        root: T,
        route: T,
    ) -> Option<Handler<P, R>> {
        if root == route {
            return self.handle;
        }

        let route: String = route.into();
        let parts = route.split(".").collect::<Vec<&str>>();

        let mut subs = self.subs.as_ref().unwrap();
        let mut handle: Option<Handler<P, R>> = None;

        for part in parts {
            if subs.contains_key(part) {
                handle = subs.get(part).unwrap().handle;
                subs = subs.get(part).unwrap().subs.as_ref().unwrap();
            } else {
                return None;
            }
        }

        handle
    }
}

impl<P, R> Router<P, R> for ServiceModule<P, R> {
    fn handle<S: Into<String>>(&mut self, submod: S, handler: Option<Handler<P, R>>) -> &Self {
        let m = Self::new(handler);

        if self.subs.is_none() {
            self.subs = Some(HashMap::new());
        }

        if let Some(subs) = self.subs.as_mut() {
            subs.insert(submod.into().to_string(), m);
        }

        self
    }
}
