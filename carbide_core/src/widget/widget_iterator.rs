use crate::widget::primitive::Widget;

pub enum WidgetIterMut<'a, S> {
    Empty,
    Single(&'a mut dyn Widget<S>, Box<WidgetIterMut<'a, S>>),
    Multi(Box<WidgetIterMut<'a, S>>, Box<WidgetIterMut<'a, S>>)
}

impl<'a, S> WidgetIterMut<'a, S> {
    pub fn single(widget: &'a mut dyn Widget<S>) -> WidgetIterMut<'a,S> {
        WidgetIterMut::Single(widget, Box::new(WidgetIterMut::Empty))
    }
}

impl<'a, S> Iterator for WidgetIterMut<'a, S> {
    type Item = &'a mut dyn Widget<S>;

    fn next(&mut self) -> Option<Self::Item> {

        let mut i = WidgetIterMut::Empty;

        std::mem::swap(self, &mut i);

        match i {
            WidgetIterMut::Empty => {
                None
            }
            WidgetIterMut::Single(n, mut b) => {
                std::mem::swap(self, &mut *b);
                Some(n)
            }
            WidgetIterMut::Multi(mut iter, mut b) => {
                match iter.next() {
                    Some(n) => {
                        std::mem::swap(self, &mut WidgetIterMut::Multi(iter, b));
                        Some(n)
                    }
                    None => {
                        std::mem::swap(self, &mut *b);
                        self.next()
                    }
                }
            }
        }
    }
}

pub enum WidgetIter<'a, S> {
    Empty,
    Single(&'a dyn Widget<S>, Box<WidgetIter<'a, S>>),
    Multi(Box<WidgetIter<'a, S>>, Box<WidgetIter<'a, S>>)
}

impl<'a, S> WidgetIter<'a, S> {
    pub fn single(widget: &'a dyn Widget<S>) -> WidgetIter<'a, S> {
        WidgetIter::Single(widget, Box::new(WidgetIter::Empty))
    }
}

impl<'a, S> Iterator for WidgetIter<'a, S> {
    type Item = &'a dyn Widget<S>;

    fn next(&mut self) -> Option<Self::Item> {

        let mut i = WidgetIter::Empty;

        std::mem::swap(self, &mut i);

        match i {
            WidgetIter::Empty => {
                None
            }
            WidgetIter::Single(n, mut b) => {
                std::mem::swap(self, &mut *b);
                Some(n)
            }
            WidgetIter::Multi(mut iter, mut b) => {
                match iter.next() {
                    Some(n) => {
                        std::mem::swap(self, &mut WidgetIter::Multi(iter, b));
                        Some(n)
                    }
                    None => {
                        std::mem::swap(self, &mut *b);
                        self.next()
                    }
                }
            }
        }
    }
}