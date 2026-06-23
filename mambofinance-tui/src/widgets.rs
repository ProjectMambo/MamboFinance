pub mod query_table;
pub mod side_bar;
pub mod user_list;

pub trait Navigable {
    type State;

    fn next_wrapped(len: usize) -> impl Fn(usize) -> usize {
        move |cur| {
            if cur >= len - 1 { 0 } else { cur + 1 }
        }
    }

    fn previous_wrapped(len: usize) -> impl Fn(usize) -> usize {
        move |cur| {
            if cur == 0 { len - 1 } else { cur - 1 }
        }
    }

    fn next(&self, state: &mut Self::State);
    fn previous(&self, state: &mut Self::State);
}

pub trait Focusable {
    fn next_clamp(len: usize, current: usize) -> usize {
        if len == 0 {
            0
        } else {
            (current + 1).min(len - 1)
        }
    }

    fn prev_clamp(current: usize) -> usize {
        current.saturating_sub(1)
    }

    fn focus_next(&mut self);
    fn focus_previous(&mut self);
}
