use std::iter;

#[derive(Debug)]
pub(crate) enum IterVariant<S, A, G, R> {
    Stack(S),
    Absolute(A),
    Grid(G),
    Repeat(R),
}

impl<S, A, G, R, T> Iterator for IterVariant<S, A, G, R>
where
    S: Iterator<Item = T>,
    A: Iterator<Item = T>,
    G: Iterator<Item = T>,
    R: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            IterVariant::Stack(s) => s.next(),
            IterVariant::Absolute(a) => a.next(),
            IterVariant::Grid(g) => g.next(),
            IterVariant::Repeat(r) => r.next(),
        }
    }
}

pub(crate) fn pre_order_iter<T, F, I>(root: T, mut children: F) -> impl Iterator<Item = T>
where
    F: FnMut(T) -> Option<I>,
    I: Iterator<Item = T>,
    T: Clone,
{
    let mut stack = Vec::with_capacity(16);
    stack.extend(children(root.clone()));
    iter::once(root).chain(iter::from_fn(move || loop {
        let current_iter = stack.last_mut()?;
        match current_iter.next() {
            Some(i) => {
                stack.extend(children(i.clone()));
                return Some(i);
            }
            None => {
                stack.pop();
            }
        }
    }))
}

#[cfg(test)]
mod tests {
    #[test]
    fn pre_order() {
        let node_children = vec![
            vec![1, 2, 3],
            vec![4, 5],
            vec![6, 7],
            vec![],
            vec![8, 9],
            vec![],
            vec![10, 11],
            vec![],
            vec![],
            vec![],
            vec![],
        ];
        let expected = vec![0, 1, 4, 8, 9, 5, 2, 6, 10, 11, 7, 3];

        let result = super::pre_order_iter(0, |i| node_children.get(i).map(|c| c.iter().copied()))
            .collect::<Vec<_>>();

        assert_eq!(result, expected);
    }
}
