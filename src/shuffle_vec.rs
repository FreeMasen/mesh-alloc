/// data structure for randomized allocation with bump-pointer speed
#[derive(Debug, Clone)]
pub struct ShuffleVector<T, Rng> {
    elements: Vec<T>,
    current: usize,
    rng: Rng,
}

impl<T, Rng> ShuffleVector<T, Rng>
where
    Rng: rand::Rng,
{
    pub fn from_parts(elements: Vec<T>, rng: Rng) -> Self {
        let mut ret = Self {
            elements,
            current: 0,
            rng,
        };
        ret.shuffle();
        ret
    }

    fn shuffle(&mut self) {
        shuffle(&mut self.elements[self.current..], &mut self.rng);
    }

    pub fn get(&mut self) -> Option<&T> {
        let ret = self.elements.get(self.current)?;
        self.current += 1;
        Some(ret)
    }

    pub fn put(&mut self, element: T) -> Option<T> {
        if self.current > 0 {
            self.current -= 1;
            self.elements[self.current] = element;
            return None;
        }
        Some(element)
    }

    pub fn len(&self) -> usize {
        self.current - self.elements.len()
    }

    pub fn is_empty(&self) -> bool {
        self.current == self.elements.len()
    }
}

fn shuffle<T, Rng>(elements: &mut [T], rng: &mut Rng)
where
    Rng: rand::Rng,
{
    for i in (1..elements.len() - 1).into_iter().rev() {
        let j = rng.random_range(0..i + 1);
        elements.swap(i, j);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shuffle_vector() {
        let rng = rand::rng();
        let mut sv = ShuffleVector::from_parts((0usize..10).collect(), rng);

        let mut allocated = Vec::new();
        for _ in 0..10 {
            allocated.push(*sv.get().unwrap());
        }

        assert_eq!(sv.get(), None);

        // Return some elements
        sv.put(allocated[0]);
        sv.put(allocated[1]);

        // Should be able to get them back
        sv.get().unwrap();
        sv.get().unwrap();
        assert_eq!(sv.get(), None);
    }
}
