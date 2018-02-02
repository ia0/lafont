// Copyright 2018 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![feature(nll)]

use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Port {
    Invalid,
    Valid { node: usize, port: usize },
}

impl Port {
    fn new(node: usize, port: usize) -> Port {
        Port::Valid { node, port }
    }

    fn extract(&self) -> (usize, usize) {
        match *self {
            Port::Invalid => panic!(),
            Port::Valid { node, port } => (node, port),
        }
    }
}

impl ::std::fmt::Display for Port {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            Port::Invalid => panic!(),
            Port::Valid { node, port } => write!(f, "({},{})", node, port),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Node {
    Construct([Port; 3]),
    Duplicate([Port; 3]),
    Erase([Port; 1]),
}

impl Node {
    fn construct() -> Node {
        Node::Construct([Port::Invalid; 3])
    }

    fn duplicate() -> Node {
        Node::Duplicate([Port::Invalid; 3])
    }

    fn erase() -> Node {
        Node::Erase([Port::Invalid; 1])
    }

    fn port(&self, p: usize) -> Port {
        match *self {
            Node::Construct(ref ports) => ports[p],
            Node::Duplicate(ref ports) => ports[p],
            Node::Erase(ref ports) => ports[p],
        }
    }

    fn port_mut(&mut self, p: usize) -> &mut Port {
        match *self {
            Node::Construct(ref mut ports) => &mut ports[p],
            Node::Duplicate(ref mut ports) => &mut ports[p],
            Node::Erase(ref mut ports) => &mut ports[p],
        }
    }
}

impl ::std::fmt::Display for Node {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            Node::Construct(ref ports) => write!(f, "{}c{}{}", ports[0], ports[1], ports[2]),
            Node::Duplicate(ref ports) => write!(f, "{}d{}{}", ports[0], ports[1], ports[2]),
            Node::Erase(ref ports) => write!(f, "{}e", ports[0]),
        }
    }
}

#[derive(Debug)]
pub struct Net {
    nodes: HashMap<usize, Node>,
    next: usize,
}

impl Net {
    pub fn new() -> Net {
        let mut net = Net {
            nodes: HashMap::new(),
            next: 0,
        };
        let a = net.create(Node::erase());
        let b = net.create(Node::erase());
        let c = net.create(Node::construct());
        let d = net.create(Node::duplicate());
        net.connect(Port::new(a, 0), Port::new(c, 1));
        net.connect(Port::new(c, 2), Port::new(d, 1));
        net.connect(Port::new(d, 2), Port::new(b, 0));
        net.connect(Port::new(c, 0), Port::new(d, 0));
        net
    }

    fn node(&self, a: usize) -> &Node {
        self.nodes.get(&a).unwrap()
    }

    fn node_mut(&mut self, a: usize) -> &mut Node {
        self.nodes.get_mut(&a).unwrap()
    }

    fn get(&self, x: Port) -> Port {
        let (node, port) = x.extract();
        self.node(node).port(port)
    }

    fn get_mut(&mut self, x: Port) -> &mut Port {
        let (node, port) = x.extract();
        self.node_mut(node).port_mut(port)
    }

    fn create(&mut self, n: Node) -> usize {
        let a = self.next;
        assert!(self.nodes.insert(a, n).is_none());
        self.next += 1;
        a
    }

    fn delete(&mut self, a: usize) {
        assert!(self.nodes.remove(&a).is_some());
    }

    fn connect(&mut self, x: Port, y: Port) {
        *self.get_mut(x) = y;
        *self.get_mut(y) = x;
    }

    fn eval_cc(&mut self, a: usize, b: usize) {
        self.connect(self.get(Port::new(a, 1)), self.get(Port::new(b, 2)));
        self.connect(self.get(Port::new(a, 2)), self.get(Port::new(b, 1)));
        self.delete(a);
        self.delete(b);
    }

    fn eval_cd(&mut self, a: usize, b: usize) {
        let c = self.create(Node::construct());
        let d = self.create(Node::duplicate());
        self.connect(self.get(Port::new(a, 1)), Port::new(d, 0));
        self.connect(self.get(Port::new(a, 2)), Port::new(b, 0));
        self.connect(self.get(Port::new(b, 1)), Port::new(a, 0));
        self.connect(self.get(Port::new(b, 2)), Port::new(c, 0));
        self.connect(Port::new(b, 1), Port::new(a, 2));
        self.connect(Port::new(b, 2), Port::new(c, 2));
        self.connect(Port::new(d, 1), Port::new(a, 1));
        self.connect(Port::new(d, 2), Port::new(c, 1));
    }

    fn eval_ce(&mut self, a: usize, b: usize) {
        let c = self.create(Node::erase());
        self.connect(Port::new(b, 0), self.get(Port::new(a, 1)));
        self.connect(Port::new(c, 0), self.get(Port::new(a, 2)));
        self.delete(a);
    }

    fn eval_dd(&mut self, a: usize, b: usize) {
        self.connect(self.get(Port::new(a, 1)), self.get(Port::new(b, 1)));
        self.connect(self.get(Port::new(a, 2)), self.get(Port::new(b, 2)));
        self.delete(a);
        self.delete(b);
    }

    fn eval_de(&mut self, a: usize, b: usize) {
        let c = self.create(Node::erase());
        self.connect(Port::new(b, 0), self.get(Port::new(a, 1)));
        self.connect(Port::new(c, 0), self.get(Port::new(a, 2)));
        self.delete(a);
    }

    fn eval_ee(&mut self, a: usize, b: usize) {
        self.delete(a);
        self.delete(b);
    }

    pub fn step(&mut self) -> bool {
        let mut cc = Vec::new();
        let mut cd = Vec::new();
        let mut ce = Vec::new();
        let mut dd = Vec::new();
        let mut de = Vec::new();
        let mut ee = Vec::new();
        for (&a, n) in self.nodes.iter() {
            let (b, p) = n.port(0).extract();
            if p == 0 && b > a {
                match (*n, *self.node(b)) {
                    (Node::Construct(_), Node::Construct(_)) => cc.push((a, b)),
                    (Node::Construct(_), Node::Duplicate(_)) => cd.push((a, b)),
                    (Node::Construct(_), Node::Erase(_)) => ce.push((a, b)),
                    (Node::Duplicate(_), Node::Construct(_)) => cd.push((b, a)),
                    (Node::Duplicate(_), Node::Duplicate(_)) => dd.push((a, b)),
                    (Node::Duplicate(_), Node::Erase(_)) => de.push((a, b)),
                    (Node::Erase(_), Node::Construct(_)) => ce.push((b, a)),
                    (Node::Erase(_), Node::Duplicate(_)) => de.push((b, a)),
                    (Node::Erase(_), Node::Erase(_)) => ee.push((a, b)),
                }
            }
        }
        if let Some((a, b)) = ee.pop() {
            self.eval_ee(a, b);
            return true;
        }
        if let Some((a, b)) = de.pop() {
            self.eval_de(a, b);
            return true;
        }
        if let Some((a, b)) = ce.pop() {
            self.eval_ce(a, b);
            return true;
        }
        if let Some((a, b)) = dd.pop() {
            self.eval_dd(a, b);
            return true;
        }
        if let Some((a, b)) = cc.pop() {
            self.eval_cc(a, b);
            return true;
        }
        if let Some((a, b)) = cd.pop() {
            self.eval_cd(a, b);
            return true;
        }
        false
    }
}

impl ::std::fmt::Display for Net {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        for (&a, n) in self.nodes.iter() {
            writeln!(f, "{}: {}", a, n)?;
        }
        writeln!(f, "{}: -", self.next)?;
        Ok(())
    }
}
