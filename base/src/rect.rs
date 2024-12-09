use crate::Rect;

impl Rect {
    pub fn take_left(&self, amount: f32) -> Self {
        Rect { x: self.x, y: self.y, w: self.w.min(amount), h: self.h }
    }

    pub fn take_top(&self, amount: f32) -> Self {
        Rect { x: self.x, y: self.y, w: self.w, h: self.h.min(amount) }
    }

    pub fn take_right(&self, amount: f32) -> Self {
        Rect {
            x: self.x.max(self.x + self.w - amount),
            y: self.y,
            w: self.w.min(amount),
            h: self.h,
        }
    }

    pub fn take_bot(&self, amount: f32) -> Self {
        Rect {
            x: self.x,
            y: self.y.max(self.y + self.h - amount),
            w: self.w,
            h: self.h.min(amount),
        }
    }

    /// the rect without specified amount of space on the left side
    pub fn skip_left(&self, amount: f32) -> Self {
        Rect { x: self.x + amount, y: self.y, w: self.w - amount, h: self.h }
    }

    /// the rect without specified amount of space on the top side
    pub fn skip_top(&self, amount: f32) -> Self {
        Rect { x: self.x, y: self.y + amount, w: self.w, h: self.h - amount }
    }

    /// the rect without specified amount of space on the right side
    pub fn skip_right(&self, amount: f32) -> Self {
        Rect { x: self.x, y: self.y, w: self.w - amount, h: self.h }
    }

    /// the rect without specified amount of space on the bottom side
    pub fn skip_bot(&self, amount: f32) -> Self {
        Rect { x: self.x, y: self.y, w: self.w, h: self.h - amount }
    }

    pub fn grow_all(&self, amount: f32) -> Self {
        Rect {
            x: self.x - amount,
            y: self.y - amount,
            w: self.w + 2. * amount,
            h: self.h + 2. * amount,
        }
    }

    /// resulting rect contains both components
    pub fn fuse(&self, other: Self) -> Self {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);

        // right end x coordinate
        let rx = (self.x + self.w).max(other.x + other.w);
        // bottom end y coordinate
        let by = (self.y + self.h).max(other.y + other.h);

        let w = rx - x;
        let h = by - y;

        Rect { x, y, w, h }
    }
}
