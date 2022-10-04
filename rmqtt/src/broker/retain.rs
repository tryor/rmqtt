type HashMap<K, V> = std::collections::HashMap<K, V, ahash::RandomState>;
type Level = ntex_mqtt::TopicLevel;
type Topic = ntex_mqtt::Topic;

pub type RetainTree<V> = Node<V>;

pub struct Node<V> {
    value: Option<V>,
    branches: HashMap<Level, Node<V>>,
}

impl<V> Default for Node<V> {
    #[inline]
    fn default() -> Node<V> {
        Self { value: None, branches: HashMap::default() }
    }
}

impl<V> Node<V>
    where
        V: std::fmt::Debug + Clone,
{
    #[inline]
    pub fn insert(&mut self, topic: &Topic, value: V) {
        let mut path = topic.levels().clone();
        path.reverse();
        self._insert(path, value);
    }

    #[inline]
    fn _insert(&mut self, mut path: Vec<Level>, value: V) {
        if let Some(first) = path.pop() {
            self.branches.entry(first).or_default()._insert(path, value)
        } else {
            self.value.replace(value);
        }
    }

    #[inline]
    pub fn remove(&mut self, topic: &Topic) -> Option<V> {
        self._remove(topic.levels().as_ref())
    }

    #[inline]
    fn _remove(&mut self, path: &[Level]) -> Option<V> {
        if path.is_empty() {
            self.value.take()
        } else {
            let t = &path[0];
            if let Some(x) = self.branches.get_mut(t) {
                let res = x._remove(&path[1..]);
                if x.value.is_none() && x.branches.is_empty() {
                    self.branches.remove(t);
                }
                res
            } else {
                None
            }
        }
    }

    #[inline]
    pub fn matches(&self, topic: &Topic) -> Vec<(Topic, V)> {
        let mut out = Vec::new();
        self._matches(topic.levels(), Vec::new(), &mut out);
        out
    }

    #[inline]
    fn _matches(&self, path: &[Level], mut sub_path: Vec<Level>, out: &mut Vec<(Topic, V)>) {
        let add_to_out = |levels: Vec<Level>, v: V, out: &mut Vec<(Topic, V)>| {
            out.push((Topic::from(levels), v));
        };

        //let node_map = &self.branches;

        if self.branches.is_empty() || path.is_empty() {
            if path.is_empty() {
                //Precise matching
                if let Some(v) = self.value.as_ref() {
                    add_to_out(sub_path, v.clone(), out);
                }
            }
        } else if !path.is_empty() {
            if let Some(r) = self.branches.get(&path[0]) {
                //Precise matching
                sub_path.push(path[0].clone());

                if path.len() > 1 && path[1] == Level::MultiWildcard {
                    //# Match parent, subscription ending with #
                    if let Some(v) = r.value.as_ref() {
                        add_to_out(sub_path.clone(), v.clone(), out);
                    }
                }
                r._matches(&path[1..], sub_path, out);
            } else if matches!(path[0], Level::SingleWildcard) {
                //Single layer matching
                for (k, v) in self.branches.iter() {
                    if sub_path.is_empty() && !matches!(k, Level::Blank) && k.is_metadata() {
                        //TopicName names starting with the $character cannot be matched with topic
                        //filters starting with wildcards (# or +)
                        continue;
                    }
                    let mut sub_path = sub_path.clone();
                    sub_path.push(k.clone());

                    if path.len() > 1 && path[1] == Level::MultiWildcard {
                        //# Match parent, subscription ending with #
                        if let Some(v) = v.value.as_ref() {
                            add_to_out(sub_path.clone(), v.clone(), out);
                        }
                    }
                    v._matches(&path[1..], sub_path, out);
                }
            } else if path[0] == Level::MultiWildcard {
                //Multilayer matching
                for (k, v) in self.branches.iter() {
                    if sub_path.is_empty() && !matches!(k, Level::Blank) && k.is_metadata() {
                        //TopicName names starting with the $character cannot be matched with topic
                        //filters starting with wildcards (# or +)
                        continue;
                    }
                    let mut sub_path = sub_path.clone();
                    sub_path.push(k.clone());

                    if v.branches.is_empty() {
                        if let Some(v) = v.value.as_ref() {
                            add_to_out(sub_path, v.clone(), out);
                        }
                    } else {
                        if let Some(v) = v.value.as_ref() {
                            add_to_out(sub_path.clone(), v.clone(), out);
                        }
                        v._matches(path, sub_path, out);
                    }
                }
            }
        }
    }

    #[inline]
    pub fn value(&self) -> Option<&V> {
        self.value.as_ref()
    }

    #[inline]
    pub fn children(&self) -> &HashMap<Level, Node<V>> {
        &self.branches
    }

    #[inline]
    pub fn child(&self, l: &Level) -> Option<&Node<V>> {
        self.branches.get(l)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::{RetainTree, Topic};

    fn match_one(tree: &RetainTree<i32>, topic_filter: &str, vs: &[i32]) -> bool {
        let mut matcheds = 0;
        let t = Topic::from_str(topic_filter).unwrap();
        for (topic, v) in tree.matches(&t).iter() {
            println!("[retain] {}({}) => {:?}, {:?}", topic_filter, topic.to_string(), v, vs);
            if !vs.contains(v) {
                return false;
            }
            matcheds += 1;
        }
        matcheds == vs.len()
    }

    #[test]
    fn retain() {
        let mut tree: RetainTree<i32> = RetainTree::default();
        tree.insert(&Topic::from_str("/iot/b/x").unwrap(), 1);
        tree.insert(&Topic::from_str("/iot/b/y").unwrap(), 2);
        tree.insert(&Topic::from_str("/iot/b/z").unwrap(), 3);
        tree.insert(&Topic::from_str("/x/y/z").unwrap(), 4);

        assert!(match_one(&tree, "/iot/b/y", &[2]));
        assert!(match_one(&tree, "/iot/b/+", &[1, 2, 3]));
        assert!(match_one(&tree, "/x/y/z", &[4]));
        assert!(!match_one(&tree, "/x/y/z", &[1]));
    }
}
