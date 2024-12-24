use std::fmt::Debug;

enum Node<T> {
    Header {
        next_index: usize,
    },
    Value {
        next_index: usize,
        pre_index: usize,
        value: T,
    },
    None {
        next_index: usize,
    },
    Tail {
        pre_index: usize,
    },
}

pub struct LinkedList<T> {
    buff: Vec<Node<T>>,
    none_index: usize,
}

impl<T> LinkedList<T> {
    const CHUNK_SIZE: usize = 1024;

    pub fn new() -> Self {
        let mut buff = Vec::with_capacity(Self::CHUNK_SIZE);
        let mut i = 2;
        buff.push(Node::Header { next_index: 1 });
        buff.push(Node::Tail { pre_index: 0 });
        buff.resize_with(Self::CHUNK_SIZE, || {
            i += 1;
            Node::None { next_index: i }
        });
        if let Some(Node::None { next_index }) = buff.last_mut() {
            // 连接到 Tail
            *next_index = 1;
        }
        Self {
            buff,
            none_index: 2,
        }
    }

    fn insert_after(&mut self, index: usize, value: T) -> Option<T> {
        if index >= self.buff.len() || self.none_index >= self.buff.len() {
            return Some(value);
        }

        let new_index = self.none_index;
        self.none_index = match &self.buff[self.none_index] {
            Node::None { next_index } => *next_index,
            _ => {
                unreachable!("none_index is invalid");
            }
        };
        let next_index = match &mut self.buff[index] {
            Node::Value { next_index, .. } => next_index,
            Node::Header { next_index } => next_index,
            _ => {
                self.none_index = new_index;
                return Some(value);
            }
        };
        let next_index_ = *next_index;
        *next_index = new_index;

        let pre_index = match &mut self.buff[next_index_] {
            Node::Value { pre_index, .. } => pre_index,
            Node::Tail { pre_index } => pre_index,
            _ => {
                unreachable!("next_index is invalid");
            }
        };
        *pre_index = new_index;

        self.buff[new_index] = Node::Value {
            next_index: next_index_,
            pre_index: index,
            value,
        };
        None
    }

    fn insert_before(&mut self, index: usize, value: T) -> Option<T> {
        if index >= self.buff.len() || self.none_index >= self.buff.len() {
            return Some(value);
        }

        let new_index = self.none_index;
        self.none_index = match &self.buff[self.none_index] {
            Node::None { next_index } => *next_index,
            _ => {
                unreachable!("none_index is invalid");
            }
        };

        let pre_index = match &mut self.buff[index] {
            Node::Value { pre_index, .. } => pre_index,
            Node::Tail { pre_index } => pre_index,
            _ => {
                self.none_index = new_index;
                return Some(value);
            }
        };

        let pre_index_ = *pre_index;
        *pre_index = new_index;

        let next_index = match &mut self.buff[pre_index_] {
            Node::Value { next_index, .. } => next_index,
            Node::Header { next_index } => next_index,
            _ => {
                unreachable!("pre_index is invalid");
            }
        };
        *next_index = new_index;

        self.buff[new_index] = Node::Value {
            next_index: index,
            pre_index: pre_index_,
            value,
        };
        None
    }

    pub fn push_back(&mut self, value: T) {
        self.insert_before(1, value);
    }

    pub fn push_head(&mut self, value: T) {
        self.insert_after(0, value);
    }

    fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.buff.len() {
            return None;
        }

        let (pre_index_, next_index_) = match &self.buff[index] {
            Node::Value {
                pre_index,
                next_index,
                ..
            } => (*pre_index, *next_index),
            _ => {
                return None;
            }
        };

        match &mut self.buff[pre_index_] {
            Node::Value { next_index, .. } => {
                *next_index = next_index_;
            }
            Node::Header { next_index } => {
                *next_index = next_index_;
            }
            _ => {
                unreachable!("pre_index is invalid");
            }
        }

        match &mut self.buff[next_index_] {
            Node::Value { pre_index, .. } => {
                *pre_index = pre_index_;
            }
            Node::Tail { pre_index } => {
                *pre_index = pre_index_;
            }
            _ => {
                unreachable!("next_index is invalid");
            }
        }

        let new_none_node = Node::None {
            next_index: self.none_index,
        };
        if let Node::Value { value, .. } = std::mem::replace(&mut self.buff[index], new_none_node) {
            self.none_index = index;
            Some(value)
        } else {
            unreachable!("index is invalid");
        }
    }

    fn get(&self, index: usize) -> Option<(&T, usize, usize)> {
        if index >= self.buff.len() {
            return None;
        }

        match &self.buff[index] {
            Node::Value {
                value,
                pre_index,
                next_index,
            } => Some((value, *pre_index, *next_index)),
            _ => None,
        }
    }

    fn get_mut(&mut self, index: usize) -> Option<(&mut T, usize, usize)> {
        if index >= self.buff.len() {
            return None;
        }

        match &mut self.buff[index] {
            Node::Value {
                value,
                pre_index,
                next_index,
            } => Some((value, *pre_index, *next_index)),
            _ => None,
        }
    }

    fn first(&self) -> Option<LinkedEntry<T>> {
        match &self.buff[0] {
            Node::Header { next_index: index } => {
                self.get(*index)
                    .map(|(_, pre_index, next_index)| LinkedEntry {
                        list: self,
                        index: *index,
                        next_index,
                        pre_index,
                    })
            }
            _ => unreachable!("Header is invalid"),
        }
    }

    fn last(&self) -> Option<LinkedEntry<T>> {
        match &self.buff[1] {
            Node::Tail { pre_index: index } => {
                self.get(*index)
                    .map(|(_, pre_index, next_index)| LinkedEntry {
                        list: self,
                        index: *index,
                        next_index,
                        pre_index,
                    })
            }
            _ => unreachable!("Tail is invalid"),
        }
    }

    fn first_mut<'a>(&'a mut self) -> Option<LinkedEntryMut<'a, T>> {
        match &self.buff[0] {
            Node::Header { next_index: index } => {
                let index = *index;
                let (_, pre_index, next_index) = self.get(index)?;
                Some(LinkedEntryMut {
                    list: self,
                    index,
                    next_index,
                    pre_index,
                })
            }
            _ => unreachable!("Header is invalid"),
        }
    }

    fn last_mut<'a>(&'a mut self) -> Option<LinkedEntryMut<'a, T>> {
        match &self.buff[1] {
            Node::Tail { pre_index: index } => {
                let index = *index;
                let (_, pre_index, next_index) = self.get(index)?;
                Some(LinkedEntryMut {
                    list: self,
                    index,
                    next_index,
                    pre_index,
                })
            }
            _ => unreachable!("Tail is invalid"),
        }
    }
}

struct LinkedEntry<'a, T> {
    list: &'a LinkedList<T>,
    index: usize,
    next_index: usize,
    pre_index: usize,
}

impl<T> LinkedEntry<'_, T> {
    pub fn next(&self) -> Option<Self> {
        self.list
            .get(self.next_index)
            .map(|(_, pre_index, next_index)| Self {
                list: self.list,
                index: self.next_index,
                pre_index,
                next_index,
            })
    }

    pub fn pre(&self) -> Option<Self> {
        let (_, p, n) = self.list.get(self.pre_index)?;
        Some(Self {
            list: self.list,
            index: self.pre_index,
            pre_index: p,
            next_index: n,
        })
    }

    pub fn value(&self) -> Option<&T> {
        self.list.get(self.index).map(|(value, _, _)| value)
    }
}

struct LinkedEntryMut<'a, T> {
    list: &'a mut LinkedList<T>,
    index: usize,
    next_index: usize,
    pre_index: usize,
}

impl<T> LinkedEntryMut<'_, T> {
    pub fn next(self) -> Option<Self> {
        let (_, p, n) = self.list.get(self.next_index)?;
        Some(Self {
            list: self.list,
            index: self.next_index,
            pre_index: p,
            next_index: n,
        })
    }

    pub fn pre(self) -> Option<Self> {
        let (_, p, n) = self.list.get(self.pre_index)?;
        Some(Self {
            list: self.list,
            index: self.pre_index,
            pre_index: p,
            next_index: n,
        })
    }

    pub fn value(&mut self) -> Option<&mut T> {
        self.list.get_mut(self.index).map(|(value, _, _)| value)
    }

    pub fn remove(self) {
        self.list.remove(self.index);
    }

    pub fn insert_after_this(self, value: T) {
        self.list.insert_after(self.index, value);
    }

    pub fn insert_before_this(self, value: T) {
        self.list.insert_before(self.index, value);
    }
}

fn print_list<T: Debug>(list: &LinkedList<T>) {
    let mut node = list.first();
    loop {
        if let Some(n) = node {
            println!("{:?}", n.value());
            node = n.next();
        } else {
            break;
        }
    }
}

fn main() {
    let mut list = LinkedList::new();
    list.push_back(1);
    list.push_back(2);
    list.push_back(3);
    print_list(&list);

    println!("after remove");
    list.first_mut().unwrap().next().unwrap().remove();
    print_list(&list);

    println!("push head");
    list.push_head(4);
    print_list(&list);

    println!("after remove");
    list.last_mut().unwrap().pre().unwrap().remove();
    print_list(&list);

    println!("insert after");
    list.first_mut().unwrap().insert_after_this(5);
    list.last_mut().unwrap().insert_before_this(6);
    print_list(&list);
}
