#![no_std]
use core::iter::{Peekable, Rev};

use heapless::consts::*;
use heapless::spsc::{Iter, Queue};

use smart_leds_trait::SmartLedsWrite;
use smart_leds_trait::RGB8;

struct QueueElement {
    color: RGB8,
    position: u16,
}

pub struct Elements {
    queue: Queue<QueueElement, U128, u16, heapless::spsc::SingleCore>,
    length: u16,
    trail_length: u16,
}

impl Elements {
    pub fn new(length: u16, trail_length: u16) -> Self {
        let queue = unsafe { Queue::u16_sc() };

        Self {
            queue,
            length,
            trail_length,
        }
    }

    pub fn step(&mut self) {
        for x in self.queue.iter_mut() {
            x.position += 1;
        }
        self.cull();
    }

    pub fn add(&mut self, color: RGB8) -> Result<(), ()> {
        let element = QueueElement { color, position: 0 };
        if self
            .queue
            .iter_mut()
            .next_back()
            .map(|x| x.position != 0)
            .unwrap_or(true)
        {
            self.queue.enqueue(element).map_err(|_| ())
        } else {
            // Too many elements, skip this one
            Ok(())
        }
    }

    // Drop elements that aren't visible anymore
    pub fn cull(&mut self) {
        while self
            .queue
            .peek()
            .map(|x| x.position > (self.length + self.trail_length))
            .unwrap_or(false)
        {
            self.queue.dequeue();
        }
    }

    pub fn iter<'a>(&'a mut self) -> ElementIter<'a> {
        ElementIter {
            iter: self.queue.iter().rev().peekable(),
            pos: 0,
            step: 255 / self.trail_length,
            trail_length: self.trail_length,
            length: self.length,
        }
    }
}

pub struct ElementIter<'a> {
    iter: Peekable<Rev<Iter<'a, QueueElement, U128, u16, heapless::spsc::SingleCore>>>,
    pos: u16,
    trail_length: u16,
    step: u16,
    length: u16,
}

impl<'a> Iterator for ElementIter<'a> {
    type Item = RGB8;
    fn next(&mut self) -> Option<RGB8> {
        let pos = self.pos;
        self.pos += 1;
        // Check if we exceeded the length
        if pos >= self.length {
            return None;
        }
        // Check if it's time for the next element
        // We don't return now to get results for the full length of the chain
        if self.iter.peek().map(|x| x.position < pos).unwrap_or(false) {
            self.iter.next();
        }
        if let Some(x) = self.iter.peek() {
            let distance = x.position - pos;
            let multiplier = self.trail_length.saturating_sub(distance) as u16 * self.step as u16;
            Some(brightness(x.color, multiplier))
        } else {
            // Return dark pixel
            Some(RGB8 { r: 0, g: 0, b: 0 })
        }
    }
}

fn brightness(color: RGB8, multiplier: u16) -> RGB8 {
    RGB8 {
        r: (color.r as u16 * multiplier / 256) as u8,
        g: (color.g as u16 * multiplier / 256) as u8,
        b: (color.b as u16 * multiplier / 256) as u8,
    }
}
