pub trait IteratorExt: Iterator {
    fn flatten(self) -> Flatten<Self>
    where
        // Self: Sized,
        Self: Sized,
        Self::Item: IntoIterator,
    {
        Flatten::new(self)
    }
}


pub fn flatten<O: IntoIterator>(iter: O) -> Flatten<O::IntoIter>
where 
    O::Item: IntoIterator,
 {
    Flatten::new(iter.into_iter())
}
pub struct Flatten<O> 
where
    O: Iterator,
    O::Item: IntoIterator,
{
    outer: O,
    front_iter: Option<<O::Item as IntoIterator>::IntoIter>,
    back_iter: Option<<O::Item as IntoIterator>::IntoIter>,
}

impl<O: Iterator> Flatten<O>
where
    O::Item: IntoIterator,
{
    pub fn new(iter: O) -> Self {
        Self { outer: iter, front_iter: None, back_iter: None }
    }
}
impl<O: Iterator> Iterator for Flatten<O>
where
    O::Item: IntoIterator,
{
    type Item = <O::Item as IntoIterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref mut front_iter) = self.front_iter {
                if let Some(v) = front_iter.next() {
                    return Some(v);
                }
                self.front_iter = None;
            }
            if let Some(next_inner) = self.outer.next() {
                self.front_iter = Some(next_inner.into_iter());
            }
            else {
                return self.back_iter.as_mut()?.next();
            }
        }
    }
}

impl<O> DoubleEndedIterator for Flatten<O>
where
    O: Iterator + DoubleEndedIterator,
    O::Item: IntoIterator,
    <O::Item as IntoIterator>::IntoIter: DoubleEndedIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref mut back_iter) = self.back_iter {
                if let Some(v) = back_iter.next_back() {
                    return Some(v);
                }
                self.back_iter = None;
            }
            if let Some(next_inner) = self.outer.next_back() {
                self.back_iter = Some(next_inner.into_iter());
            }
            else {
                return self.front_iter.as_mut()?.next_back();
            }
        }
    }
}









#[cfg(test)]
mod tests {
    use super::*;

    
    
}
