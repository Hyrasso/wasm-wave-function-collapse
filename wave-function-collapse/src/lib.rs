use std::collections::{HashMap, HashSet};
use std::cmp::Ordering;
use std::sync::Mutex;

use wasm_bindgen::prelude::*;
// use serde::Serialize;

// trait WFC<Cell> {
//     fn step() -> bool;
//     fn collapsed() -> Vec<Cell>;
// }

static INSTANCE: Mutex<Option<WFC<[i64; 2], Vec<f64>>>> = Mutex::new(None);

#[wasm_bindgen]
pub fn init_instance() {
    let mut inst = INSTANCE.lock().unwrap();
    if inst.is_none() {
        // ' #' '##' '# ' '  '
        let constraints_list = vec![
            (0, 0, -1, 1),
            (0, 0, -1, 0),
            (0, 0, 1, 0),
            (0, 0, 1, 3),
            (1, 0, -1, 2),
            (1, 0, 1, 0),
            (1, 0, -1, 3),
            (1, 0, 1, 3),
            (2, 0, -1, 2),
            (2, 0, 1, 1),
            (2, 0, 1, 2),
            (2, 0, -1, 3),
            (3, 0, -1, 0),
            (3, 0, -1, 1),
            (3, 0, 1, 1),
            (3, 0, 1, 2),
        ];
        let constraints = generate_constraints(constraints_list);
        *inst = Some(WFC::new(constraints, vec![1.0, 1.0, 1.0, 10.0], 42));
    }
}


#[wasm_bindgen]
pub fn step() -> bool {
    let mut inst = INSTANCE.lock().unwrap();
    let res = inst.as_mut().map(|inst| inst.step());
    res.unwrap_or(false)
}

#[wasm_bindgen]
pub fn read_state() -> JsValue {
    let mut inst = INSTANCE.lock().unwrap();
    let ret_val = if let Some(ref mut inst) = *inst {
        inst.collapsed.iter().map(|(k, v)| {let mut r = k.to_vec();r.push(*v as i64);r}).collect()
    } else {
        vec![]
    };
    serde_wasm_bindgen::to_value(&ret_val).unwrap()
    // JsValue::from_serde(&ret_val).unwrap()
}

struct WFC<Pos, State> {
    state_weights: Vec<f64>,
    wavefront: HashMap<Pos, State>,
    collapsed: HashMap<Pos, usize>,
    /// Forbidden neighbors: tile, axis, dir : forbidden tiles
    constraints: HashMap<(usize, usize, i64), HashSet<usize>>,
    random_state: usize,
}

fn plogp(p: f64) -> f64 {
    if p == 0.0 {
        0.0
    } else {
        p * p.log2()
    }
}


enum UpdateState { Updated, ImpossibleState, NothingToDo }

fn generate_constraints(constraints: Vec<(usize, usize, i64, usize)>) -> HashMap<(usize, usize, i64), HashSet<usize>> {
    let mut cm = HashMap::new();
    for (tid, axis, dir, ntid) in constraints {
        cm.entry((tid, axis, dir)).or_insert(HashSet::new()).insert(ntid);
    }
    cm
}

impl WFC<[i64; 2], Vec<f64>> {
    fn new(constraints: HashMap<(usize, usize, i64), HashSet<usize>>, state_weights: Vec<f64>, random_state: usize) -> Self {
        let weight_sum: f64 = state_weights.iter().sum();
        let weight_distrib = state_weights.iter().map(|val| val / weight_sum).collect();
        WFC {
            constraints,
            state_weights: weight_distrib,
            wavefront: HashMap::new(),
            collapsed: HashMap::new(),
            random_state
        }
    }

    fn init_state(&self) -> Vec<f64> {
        vec![1.0; self.state_weights.len()]
    }

    fn state_from_idx(&self, idx: usize) -> Vec<f64> {
        let mut state = vec![0.0; self.state_weights.len()];
        state[idx] = 1.0;
        state
    }

    fn rng_next(&mut self) -> usize {
        let a = 1664525;
        let c = 1013904223;
        let m = 0xFFFFFFFF; // 2**32
        self.random_state = self.random_state.wrapping_mul(a).wrapping_add(c) % m;
        return self.random_state;
    }

    fn select_random_tile(&mut self, possible_idx: Vec<usize>) -> usize {
        // assumes possible_idx are unique
        let mut n = (self.rng_next() as f64) / (0xFFFFFFFFi64 as f64);
        let distrib: Vec<f64> = possible_idx.iter().map(|i| self.state_weights[*i]).collect();
        let distrib_sum: f64 = distrib.iter().sum();
        let norm_distrib = distrib.iter().map(|p| *p / distrib_sum);
        for (idx, p) in norm_distrib.enumerate() {
            if n < p {
                return possible_idx[idx];
            }
            n -= p;
        }
        possible_idx[possible_idx.len() - 1]
    }

    fn entropy(&self, array: &Vec<f64>) -> f64 {
        array.iter().zip(self.state_weights.iter()).map(|(state, weight)| state * weight).map(plogp).sum::<f64>() * -1.0
    }

    fn collapse(&mut self, pos: [i64; 2]) -> bool {
        if self.collapsed.contains_key(&pos) {
            return true;
        }
        let state = self.state_at(&pos);
        self.wavefront.remove(&pos);
        let allowed_idx: Vec<_> = state.iter().enumerate().filter_map(|(idx, val)| if *val > 0.0 {Some(idx)} else {None} ).collect();
        if allowed_idx.len() == 0 {
            return false;
        }
        // super random idx choice
        let tile_idx = self.select_random_tile(allowed_idx);
        // TODO: take weights into account
        // alias method, or compute cdf, a random 0-1 and pick idx where cdfi > random
        self.collapsed.insert(pos.clone(), tile_idx);
        self.propagate(pos)
    }

    fn neighbors(&self) -> Vec<(usize, i64)> {
        let mut ns = Vec::with_capacity(self.state_weights.len() * 2);
        for i in 0..self.state_weights.len() {
            ns.push((i, 1));
            ns.push((i, -1));
        }
        return ns;
    }

    fn state_at(&self, pos: &[i64; 2]) -> Vec<f64> {
        match self.collapsed.get(pos) {
            Some(tile_idx) => self.state_from_idx(*tile_idx),
            None => match self.wavefront.get(pos) {
                None => self.init_state(),
                Some(state) => state.clone()
            }
        }
    }

    fn set_state_at(&mut self, pos: [i64; 2], state: Vec<f64>) {
        let mut allowed_tiles = state.iter().enumerate().filter_map(|(idx, v)| if *v > 0.0 { Some(idx) } else { None });
        let tile_id = allowed_tiles.next();
        // if tile_id is none that means the provided state has 0 allowed state
        if allowed_tiles.next().is_none() && tile_id.is_some() {
            self.collapsed.insert(pos, tile_id.unwrap());
        } else {
            self.wavefront.insert(pos, state);
        }
    }

    fn exclude(&mut self, forbidden_states: HashSet<usize>, neighbor: [i64; 2]) -> UpdateState {
        let mut state = self.state_at(&neighbor);
        let mut updated = false;
        for i in forbidden_states {
            if state[i] > 0.0 {
                state[i] = 0.0;
                updated = true;
            }
        }
        if !updated {
            return UpdateState::NothingToDo;
        }
        if state.iter().all(|s| *s == 0.0) {
            return UpdateState::ImpossibleState;
        }
        self.set_state_at(neighbor, state);
        UpdateState::Updated
    }

    fn update_neighbor(&mut self, pos: &[i64; 2], dpos: (usize, i64)) -> UpdateState {
        let current_state = self.state_at(pos);
        let (axis, dir) = dpos;
        let mut neighbor_forbidden: Option<HashSet<usize>> = None;
        for (i, val) in current_state.iter().enumerate() {
            if *val > 0.0 {
                if let Some(forbidden_tiles) = self.constraints.get(&(i, axis, dir)) {
                    neighbor_forbidden = match neighbor_forbidden {
                        None => Some(forbidden_tiles.clone()),
                        Some(nf) => Some(forbidden_tiles.clone().intersection(&nf).copied().collect::<HashSet<usize>>())
                    }
                }
            }
        }
        match neighbor_forbidden {
            None => UpdateState::NothingToDo,
            Some(forbidden_states) if forbidden_states.len() == 0 => UpdateState::NothingToDo,
            Some(forbidden_states) => {
                let mut neighbor = pos.clone();
                neighbor[dpos.0] += dpos.1;
                self.exclude(forbidden_states.clone(), neighbor)
            }
        }
    }

    // returns false if the propagation resulted in an impossible state
    fn propagate(&mut self, pos: [i64; 2]) -> bool {
        let mut stack = vec![pos];

        while let Some(pos) = stack.pop() {
            // update the state of all neighbors
            for neighbors_dpos in self.neighbors() {
                // update neighbors states if necessary
                match self.update_neighbor(&pos, neighbors_dpos) {
                    UpdateState::Updated => {
                        let mut neighbor = pos.clone();
                        neighbor[neighbors_dpos.0] += neighbors_dpos.1;
                        stack.push(neighbor.clone());
                    },
                    UpdateState::ImpossibleState => return false,
                    UpdateState::NothingToDo => ()
                }
            }
        }
        true
    }

    fn step(&mut self) -> bool {
        // get the min entropy cell
        let min_h_cell = self.wavefront.iter().min_by(|a, b|
            match f64::total_cmp(&self.entropy(a.1), &self.entropy(b.1)) {
                Ordering::Greater => Ordering::Greater,
                Ordering::Less => Ordering::Less,
                Ordering::Equal => i64::cmp(&a.0.iter().map(|v| v.abs()).max().unwrap(), &b.0.iter().map(|v| v.abs()).max().unwrap())
            });
        let pos = match min_h_cell{
            Some(value) => value.0,
            // TODO: get a the closest tile to 0,0 that is not collapsed
            None => &[0, 0],
        };
        let pos = pos.clone();

        // collapse the cell and propagate
        return self.collapse(pos);
    }
}


#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;

    fn simple_constraints() -> Vec<(usize, usize, i64, usize)> {
        // dummy example with one axis and 3 tiles (' #' '##' '# ')
        return vec![
            (0, 0, -1, 1),
            (0, 0, -1, 0),
            (0, 0, 1, 0),
            (1, 0, -1, 2),
            (1, 0, 1, 0),
            (2, 0, -1, 2),
            (2, 0, 1, 1),
            (2, 0, 1, 2),
        ];
    }

    fn simple_init() -> WFC<[i64; 2], Vec<f64>> {
        let constraint_list = simple_constraints();
        let constraints = generate_constraints(constraint_list);
        let wfc = WFC::new(constraints, vec![1.0, 1.0, 1.0], 42);
        
        return wfc;
    }

    #[test]
    fn gen_contraints() {
        let contraints_list = simple_constraints();
        let constraints = generate_constraints(contraints_list);
        assert_eq!(constraints.get(&(0, 0, -1)), Some(&HashSet::from([0, 1])));
        assert_eq!(constraints.get(&(0, 0, 1)), Some(&HashSet::from([0])));
        assert!(constraints.get(&(0, 1, 1)).is_none());
    }

    #[test]
    fn init_wfc() {
        let wfc = simple_init();

        assert_eq!(wfc.collapsed.len(), 0);
        assert_eq!(wfc.wavefront.len(), 0);
        assert_eq!(wfc.constraints.len(), 6);
    }

    #[test]
    fn logprob_f() {
        assert_eq!(plogp(0.0), 0.0);
        assert_eq!(plogp(1.0), 0.0);
    }

    #[test]
    fn state_for_index() {
        let wfc = simple_init();
        let state = wfc.state_from_idx(0);
        assert!(state.len() == 3);
        assert!(state[0] == 1.0);
        assert!(state[1] == 0.0);
        assert!(state[2] == 0.0);
    }

    #[test]
    fn entropy() {
        let wfc = simple_init();

        let state = vec![1.0, 1.0, 1.0];
        let target = f64::log2(1.0/3.0) * -3.0;
        assert!(wfc.entropy(&state) - target < 1e6);
        let state2 = vec![1.0, 1.0, 0.0];
        assert!(wfc.entropy(&state) > wfc.entropy(&state2));
    }

    #[test]
    fn collapse() {
        let mut wfc = simple_init();
        let r = wfc.collapse([0, 0]);
        assert!(r);
    }

    #[test]
    fn step_n() {
        let mut wfc = simple_init();
        assert!(wfc.step());
        assert!(wfc.step());
        assert!(wfc.step());
        assert!(wfc.step());
        assert!(wfc.step());
        // remove this assert, the only constant is the wavefront size
        // assert_eq!(wfc.collapsed.len(), 6);
        assert_eq!(wfc.wavefront.len(), 2);
    }
}
