// Copyright (C) 2021 Marcus Xu
//
// This file is part of minesweeper.
//
// minesweeper is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// minesweeper is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with minesweeper.  If not, see <http://www.gnu.org/licenses/>.

use super::*;

pub struct Solver(Vec<Square>);

impl Solver {
    
}

fn lift<T, F: Fn(T, T) -> T>(a: Option<T>, b: Option<T>, f: F) -> Option<T> {
    match (a, b) {
        (Some(x), Some(y)) => Some(f(x, y)),
        (a, b) => a.or(b),
    }
}

fn square_of<'a, F: 'static + Fn(Status) -> bool>(config: &'a Config, state: &'a MinesweeperState, idx: Index, f: F) -> impl Iterator<Item = Index> + 'a {
    config.square(idx).iter()
        .filter_map(move |&cidx| f(state.get(cidx)).then(|| cidx))
}

pub fn solve_next(config: &Config, state: &mut MinesweeperState) -> Option<ScoredIndex> {
    center_search(config, state)
        .or_else(|| fast_search(state))
        .or_else(|| {
            make_consistent_all(config, state);
            fast_search(state)
        })
        .or_else(|| {
            let mut eval = None;
            let mut group = Group::new(state);
            for idx in 0..config.size() {
                let (group_, subset) = group.split(idx, config);
                group = group_;
                if !subset.is_empty() {
                    let unknowns = state.unknowns();
                    let flags_remaining = state.flags_remaining();
                    let binop = |e_0: Evaluation, e_1: Evaluation| {
                        Evaluation::add_disj_group_evals(unknowns, flags_remaining, e_0, e_1)
                    };
                    eval = lift(eval, evaluate(config, state.clone(), subset), binop)
                }
            }
            let eval = eval?;
            eval.label_certains(state);
            println!("{:?}", eval);
            fast_search(state)
        })
}

fn make_consistent(idx: Index, config: &Config, state: &mut MinesweeperState) -> bool {
    if let Some(count) = state.get_known(idx) {
        let minimum = square_of(config, state, idx, |status| status == Status::Flagged).count();
        let maximum = square_of(config, state, idx, |status| status == Status::Unknown).count() + minimum;
        if !(minimum..=maximum).contains(&count) {
            return false;
        }
        let square = square_of(config, state, idx, |status| status == Status::Unknown).collect::<Square>();
        square.into_iter().all(|cidx| {
            if count == minimum && state.get(cidx) != Status::Marked {
                state.set_mark(cidx);
            } else if count == maximum && state.get(cidx) != Status::Flagged {
                state.set_flag(cidx);
            } else {
                return true;
            }
            make_consistent_sq(cidx, config, state)
        })
    } else {
        true
    }
}

fn make_consistent_sq(idx: Index, config: &Config, state: &mut MinesweeperState) -> bool {
    config.square(idx).iter().all(|&cidx| make_consistent(cidx, config, state))
}

fn make_consistent_all(config: &Config, state: &mut MinesweeperState) -> bool {
    (0..config.size()).all(|cidx| make_consistent(cidx, config, state))
}

fn recur(config: &Config, mut state: MinesweeperState, idx: Index, subset_0: Group, subset_1: Group) -> Option<Evaluation> {
    if !make_consistent(idx, config, &mut state) {
        return None;
    }
    let unknowns = state.unknowns();
    let flags_remaining = state.flags_remaining();
    let binop = |e_0: Evaluation, e_1: Evaluation| {
        Evaluation::add_disj_group_evals(unknowns, flags_remaining, e_0, e_1)
    };
    let e = Evaluation::new(&state, idx);
    let e_0 = evaluate(config, state.clone(), subset_0)?;
    let e_1 = evaluate(config, state, subset_1)?;
    Some(binop(binop(e_0, e_1), e))
}

fn evaluate(config: &Config, state: MinesweeperState, group: Group) -> Option<Evaluation> {
    let idx = group.get(&state)?;  // TODO
    let (subset_0, subset_1) = group.split(idx, config);
    let mut state_to_flag = state.clone();
    let mut state_to_mark = state; // move
    state_to_flag.set_flag(idx);
    state_to_mark.set_mark(idx);
    let e_0 = recur(config, state_to_flag, idx, subset_0.clone(), subset_1.clone());
    let e_1 = recur(config, state_to_mark, idx, subset_0, subset_1);
    lift(e_0, e_1, Evaluation::add_same_group_evals)
}

fn center_search(config: &Config, state: &MinesweeperState) -> Option<ScoredIndex> {
    let center = config.center();
    match state.get(center) {
        Status::Known(_) => None,
        Status::Flagged | Status::Marked => unreachable!(),
        _ => Some((R64::new(0.0), center)),
    }
}

fn fast_search(state: &MinesweeperState) -> Option<ScoredIndex> {
    let idx = state
        .board()
        .iter()
        .position(|status| status == &Status::Marked)?;
    Some((R64::new(0.0), idx))
}

/*
impl<T: Sized + Minesweeper> Iterator for Solver<T> {
    type Item = MsResult<ScoredIndex>;
    fn next(&mut self) -> Option<Self::Item> {
        self.solve_next().transpose()
    }
}
*/
