// Copyright 2018-2022 Google LLC
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

use getopts::Options;
use kiss3d::camera::ArcBall;
use kiss3d::light::Light;
use kiss3d::nalgebra::core::Vector3;
use kiss3d::nalgebra::geometry::{Point3, Translation3};
use kiss3d::scene::SceneNode;
use kiss3d::window::Window;
use rand::random;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Port {
    agent: usize,
    port: usize,
}

impl Port {
    fn new(agent: usize, port: usize) -> Port {
        Port { agent, port }
    }
}

#[derive(Clone, Copy, Debug)]
enum Agent {
    Construct([Option<Port>; 3]),
    Duplicate([Option<Port>; 3]),
    Erase([Option<Port>; 1]),
}

struct Node {
    agent: Agent,
    scene: SceneNode,
    velocity: Vector3<f32>,
}

impl Node {
    fn get_port(&self, p: usize) -> Port {
        match self.agent {
            Agent::Construct(ref ports) => ports[p].unwrap(),
            Agent::Duplicate(ref ports) => ports[p].unwrap(),
            Agent::Erase(ref ports) => ports[p].unwrap(),
        }
    }

    fn set_port(&mut self, p: usize, x: Port) {
        match self.agent {
            Agent::Construct(ref mut ports) => ports[p] = Some(x),
            Agent::Duplicate(ref mut ports) => ports[p] = Some(x),
            Agent::Erase(ref mut ports) => ports[p] = Some(x),
        }
    }

    fn position(&self) -> Vector3<f32> {
        self.scene.data().local_translation().vector
    }
}

struct Net {
    window: Window,
    camera: ArcBall,
    nodes: HashMap<usize, Node>,
    next: usize,
}

impl Net {
    fn new() -> Net {
        let mut net = Net {
            window: Window::new("Lafont"),
            camera: ArcBall::new(Point3::new(100., 0., 0.), Point3::origin()),
            nodes: HashMap::new(),
            next: 0,
        };
        net.window.set_light(Light::StickToCamera);
        net
    }

    fn node(&self, a: usize) -> &Node {
        self.nodes.get(&a).unwrap()
    }

    fn node_mut(&mut self, a: usize) -> &mut Node {
        self.nodes.get_mut(&a).unwrap()
    }

    fn get_port(&self, x: Port) -> Port {
        self.node(x.agent).get_port(x.port)
    }

    fn set_port(&mut self, x: Port, y: Port) {
        self.node_mut(x.agent).set_port(x.port, y);
    }

    fn create(&mut self, agent: Agent) -> usize {
        let a = self.next;
        let scene = self.window.add_sphere(1.);
        let velocity = Vector3::zeros();
        let n = Node {
            agent,
            scene,
            velocity,
        };
        assert!(self.nodes.insert(a, n).is_none());
        self.next += 1;
        a
    }

    fn create_construct(&mut self) -> usize {
        let a = self.create(Agent::Construct([None; 3]));
        self.node_mut(a).scene.set_color(0., 0., 1.);
        a
    }

    fn create_duplicate(&mut self) -> usize {
        let a = self.create(Agent::Duplicate([None; 3]));
        self.node_mut(a).scene.set_color(0., 1., 0.);
        a
    }

    fn create_erase(&mut self) -> usize {
        let a = self.create(Agent::Erase([None; 1]));
        self.node_mut(a).scene.set_color(1., 0., 0.);
        a
    }

    fn delete(&mut self, a: usize) {
        self.node_mut(a).scene.unlink();
        assert!(self.nodes.remove(&a).is_some());
    }

    fn connect(&mut self, x: Port, y: Port) {
        self.set_port(x, y);
        self.set_port(y, x);
    }

    fn eval_cc(&mut self, a: usize, b: usize) {
        self.connect(
            self.get_port(Port::new(a, 1)),
            self.get_port(Port::new(b, 2)),
        );
        self.connect(
            self.get_port(Port::new(a, 2)),
            self.get_port(Port::new(b, 1)),
        );
        self.delete(a);
        self.delete(b);
    }

    fn eval_cd(&mut self, a: usize, b: usize, t: Translation3<f32>) {
        let c = self.create_construct();
        let d = self.create_duplicate();
        self.node_mut(c).scene.append_translation(&t);
        self.node_mut(d).scene.append_translation(&t);
        self.connect(self.get_port(Port::new(a, 1)), Port::new(d, 0));
        self.connect(self.get_port(Port::new(a, 2)), Port::new(b, 0));
        self.connect(self.get_port(Port::new(b, 1)), Port::new(a, 0));
        self.connect(self.get_port(Port::new(b, 2)), Port::new(c, 0));
        self.connect(Port::new(b, 1), Port::new(a, 2));
        self.connect(Port::new(b, 2), Port::new(c, 2));
        self.connect(Port::new(d, 1), Port::new(a, 1));
        self.connect(Port::new(d, 2), Port::new(c, 1));
    }

    fn eval_ce(&mut self, a: usize, b: usize, t: Translation3<f32>) {
        let c = self.create_erase();
        self.node_mut(c).scene.append_translation(&t);
        self.connect(Port::new(b, 0), self.get_port(Port::new(a, 1)));
        self.connect(Port::new(c, 0), self.get_port(Port::new(a, 2)));
        self.delete(a);
    }

    fn eval_dd(&mut self, a: usize, b: usize) {
        self.connect(
            self.get_port(Port::new(a, 1)),
            self.get_port(Port::new(b, 1)),
        );
        self.connect(
            self.get_port(Port::new(a, 2)),
            self.get_port(Port::new(b, 2)),
        );
        self.delete(a);
        self.delete(b);
    }

    fn eval_de(&mut self, a: usize, b: usize, t: Translation3<f32>) {
        let c = self.create_erase();
        self.node_mut(c).scene.append_translation(&t);
        self.connect(Port::new(b, 0), self.get_port(Port::new(a, 1)));
        self.connect(Port::new(c, 0), self.get_port(Port::new(a, 2)));
        self.delete(a);
    }

    fn eval_ee(&mut self, a: usize, b: usize) {
        self.delete(a);
        self.delete(b);
    }

    fn step(&mut self) {
        let mut collisions = Vec::new();
        for (&a, n) in self.nodes.iter() {
            let Port { agent: b, port: p } = n.get_port(0);
            if p == 0 && b > a && (n.position() - self.node(b).position()).norm() < 0.1 {
                collisions.push((a, b));
            }
        }
        for (a, b) in collisions {
            let t = Translation3::from((self.node(a).position() + self.node(a).position()) / 2.);
            match (self.node(a).agent, self.node(b).agent) {
                (Agent::Construct(_), Agent::Construct(_)) => self.eval_cc(a, b),
                (Agent::Construct(_), Agent::Duplicate(_)) => self.eval_cd(a, b, t),
                (Agent::Construct(_), Agent::Erase(_)) => self.eval_ce(a, b, t),
                (Agent::Duplicate(_), Agent::Construct(_)) => self.eval_cd(b, a, t),
                (Agent::Duplicate(_), Agent::Duplicate(_)) => self.eval_dd(a, b),
                (Agent::Duplicate(_), Agent::Erase(_)) => self.eval_de(a, b, t),
                (Agent::Erase(_), Agent::Construct(_)) => self.eval_ce(b, a, t),
                (Agent::Erase(_), Agent::Duplicate(_)) => self.eval_de(b, a, t),
                (Agent::Erase(_), Agent::Erase(_)) => self.eval_ee(a, b),
            }
        }
        let mut accelerations = HashMap::new();
        for (&a, n) in self.nodes.iter() {
            let mut acceleration = -0.1 * n.velocity;
            for (&b, m) in self.nodes.iter() {
                if a == b {
                    continue;
                }
                let mut force = n.position() - m.position();
                let distance = force.normalize_mut();
                if (n.get_port(0) == Port { agent: b, port: 0 }) {
                    if distance < 0.1 {
                        force = Vector3::zeros();
                    } else {
                        force *= -0.1;
                        match (n.agent, m.agent) {
                            (Agent::Construct(_), Agent::Duplicate(_))
                            | (Agent::Duplicate(_), Agent::Construct(_)) => force *= 0.67,
                            (Agent::Erase(_), Agent::Erase(_)) => force *= 1.5,
                            (_, _) => (),
                        }
                    }
                } else if distance < 1. {
                    let rand = || 0.5 - random::<f32>();
                    force = Vector3::new(rand(), rand(), rand()).normalize();
                    force *= 10.;
                } else if distance < 10. {
                    force /= distance * distance;
                } else {
                    force = Vector3::zeros();
                }
                acceleration += force;
            }
            assert!(accelerations.insert(a, acceleration).is_none());
        }
        for (&a, n) in self.nodes.iter_mut() {
            n.velocity += accelerations.get(&a).unwrap();
            n.scene
                .append_translation(&Translation3::from(0.1 * n.velocity));
        }
    }

    fn execute(&mut self, n: i32, v: bool) {
        assert!(n > 0);
        let mut next = Instant::now() + Duration::from_secs(10);
        let mut count = 0;
        while self.window.render_with_camera(&mut self.camera) {
            if Instant::now() < next {
                count += 1;
            } else {
                println!("fps={}.{}", count / 10, count % 10);
                next += Duration::from_secs(10);
                count = 0;
            }
            for _ in 0..n {
                self.step();
            }
            if v {
                for (&a, n) in self.nodes.iter() {
                    let Port { agent: b, port: p } = n.get_port(0);
                    let mut color = Vector3::new(1., 1., 1.);
                    if p != 0 {
                        color *= 0.3;
                    } else if b > a {
                        continue;
                    }
                    self.window.draw_line(
                        &Point3::from(n.position()),
                        &Point3::from(self.node(b).position()),
                        &Point3::from(color),
                    );
                }
            }
        }
    }
}

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let mut opts = Options::new();
    opts.optopt("n", "", "speed factor", "N");
    opts.optflag("v", "", "show principal edges");
    let matches = match opts.parse(&args) {
        Ok(m) => m,
        Err(f) => panic!("{}", f),
    };
    let v = matches.opt_present("v");
    let n = matches
        .opt_str("n")
        .map(|x| x.parse().unwrap())
        .unwrap_or(1);
    assert!(matches.free.is_empty());

    let mut net = Net::new();
    let a = net.create_erase();
    let b = net.create_erase();
    let c = net.create_construct();
    let d = net.create_duplicate();
    net.connect(Port::new(a, 0), Port::new(c, 1));
    net.connect(Port::new(c, 2), Port::new(d, 1));
    net.connect(Port::new(d, 2), Port::new(b, 0));
    net.connect(Port::new(c, 0), Port::new(d, 0));
    net.execute(n, v);
}
