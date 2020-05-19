use chrono::{NaiveDate, Duration};
use std::ops::{Add, Sub};

#[derive(PartialEq, PartialOrd, Clone)]
pub struct NaiveDateEx
{
    date: chrono::NaiveDate
}

impl Into<NaiveDate> for NaiveDateEx
{
    fn into(self) -> chrono::NaiveDate
    { 
        self.date
    }
}

impl From<NaiveDate> for NaiveDateEx
{
    fn from(from: chrono::NaiveDate) -> Self
    { 
        Self {
            date : from
        }
    }
}

impl std::iter::Step for NaiveDateEx
{
    fn steps_between(_start: &Self, _end: &Self) -> std::option::Option<usize> 
    { 
        todo!() 
    }

    fn replace_one(&mut self) -> Self 
    { 
        todo!() 
    }

    fn replace_zero(&mut self) -> Self 
    {
        todo!() 
    }

    fn add_one(&self) -> Self 
    { 
        let dur = Duration::days(1);
        return NaiveDateEx::from(self.date.clone().add(dur));
    }
    
    fn sub_one(&self) -> Self 
    { 
        let dur = Duration::days(1);
        return NaiveDateEx::from(self.date.clone().sub(dur));
    }

    fn add_usize(&self, days: usize) -> std::option::Option<Self> 
    { 
        use std::convert::TryInto;
        let dur = Duration::days(days.try_into().unwrap());
        return Some(NaiveDateEx::from(self.date.clone().add(dur)));
    }
}