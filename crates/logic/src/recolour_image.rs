use core::marker::PhantomData;

use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Dimensions, OriginDimensions, Point, Size},
    image::{GetPixel, ImageDrawable},
    iterator::raw::RawDataSlice,
    pixelcolor::{
        raw::{BigEndian, ByteOrder, RawData},
        Gray8, PixelColor,
    },
    primitives::Rectangle,
};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct RecolouredImageRaw<'a, C, BO = BigEndian>
where
    C: PixelColor + From<<C as PixelColor>::Raw>,
    BO: ByteOrder,
{
    /// Image data, packed as dictated by raw data type `Gray8::Raw`
    data: &'a [u8],

    /// Image size in pixels
    size: Size,

    pixel_type: PhantomData<C>,
    byte_order: PhantomData<BO>,

    recolor_to: (C, C),
}

impl<'a, C, BO> RecolouredImageRaw<'a, C, BO>
where
    C: PixelColor + From<<C as PixelColor>::Raw>,
    BO: ByteOrder,
{
    pub const fn new(data: &'a [u8], width: u32, recolor_to: (C, C)) -> Self {
        // Prevent panic for `width == 0` by returning a zero sized image.
        if width == 0 {
            return Self {
                data: &[],
                size: Size::zero(),
                pixel_type: PhantomData,
                byte_order: PhantomData,
                recolor_to,
            };
        }

        let height = data.len() / bytes_per_row(width, <Gray8 as PixelColor>::Raw::BITS_PER_PIXEL);

        Self {
            data,
            size: Size::new(width, height as u32),
            pixel_type: PhantomData,
            byte_order: PhantomData,
            recolor_to,
        }
    }

    const fn data_width(&self) -> u32 {
        self.size.width
        // let pixels_per_byte = 8 / <Gray8 as PixelColor>::Raw::BITS_PER_PIXEL as u32;

        // bytes_per_row(
        //     self.size.width,
        //     <Gray8 as PixelColor>::Raw::BITS_PER_PIXEL,
        // ) as u32
        // * pixels_per_byte
    }
}

/// Returns the length of each row in bytes.
const fn bytes_per_row(width: u32, bits_per_pixel: usize) -> usize {
    (width as usize * bits_per_pixel + 7) / 8
}

impl<'a, C, BO> ImageDrawable for RecolouredImageRaw<'a, C, BO>
where
    C: PixelColor + From<<C as PixelColor>::Raw>,
    BO: ByteOrder,
    RawDataSlice<'a, <Gray8 as PixelColor>::Raw, BO>:
        IntoIterator<Item = <Gray8 as PixelColor>::Raw>,
{
    type Color = C;

    fn draw<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        let row_skip = self.data_width() - self.size.width;

        target.fill_contiguous(
            &self.bounding_box(),
            ContiguousPixels::new(self, self.size, 0, row_skip as usize, self.recolor_to),
        )
    }

    fn draw_sub_image<D>(&self, target: &mut D, area: &Rectangle) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        // Don't draw anything if `area` is zero sized or partially outside the image.
        if area.is_zero_sized()
            || area.top_left.x < 0
            || area.top_left.y < 0
            || area.top_left.x as u32 + area.size.width > self.size.width
            || area.top_left.y as u32 + area.size.height > self.size.height
        {
            return Ok(());
        }

        let data_width = self.data_width() as usize;

        let initial_skip = area.top_left.y as usize * data_width + area.top_left.x as usize;
        let row_skip = data_width - area.size.width as usize;

        target.fill_contiguous(
            &Rectangle::new(Point::zero(), area.size),
            ContiguousPixels::new(self, area.size, initial_skip, row_skip, self.recolor_to),
        )
    }
}

impl<C, BO> OriginDimensions for RecolouredImageRaw<'_, C, BO>
where
    C: PixelColor + From<<C as PixelColor>::Raw>,
    BO: ByteOrder,
{
    fn size(&self) -> Size {
        self.size
    }
}

impl<'a, C, BO> GetPixel for RecolouredImageRaw<'a, C, BO>
where
    C: PixelColor + From<<C as PixelColor>::Raw>,
    BO: ByteOrder,
    RawDataSlice<'a, <Gray8 as PixelColor>::Raw, BO>:
        IntoIterator<Item = <Gray8 as PixelColor>::Raw>,
{
    type Color = C;

    fn pixel(&self, p: Point) -> Option<Self::Color> {
        if p.x < 0 || p.y < 0 || p.x >= self.size.width as i32 || p.y >= self.size.height as i32 {
            return None;
        }

        RawDataSlice::new(self.data)
            .into_iter()
            .nth(p.x as usize + p.y as usize * self.data_width() as usize)
            .map(|r| {
                if r.into_inner() > 0 {
                    self.recolor_to.1
                } else {
                    self.recolor_to.0
                }
            })
    }
}

struct ContiguousPixels<'a, C, BO>
where
    C: PixelColor,
    BO: ByteOrder,
    RawDataSlice<'a, <Gray8 as PixelColor>::Raw, BO>:
        IntoIterator<Item = <Gray8 as PixelColor>::Raw>,
{
    iter: <RawDataSlice<'a, <Gray8 as PixelColor>::Raw, BO> as IntoIterator>::IntoIter,

    remaining_x: u32,
    width: u32,

    remaining_y: u32,
    row_skip: usize,

    recolor_to: (C, C),
}

impl<'a, C, BO> ContiguousPixels<'a, C, BO>
where
    C: PixelColor + From<<C as PixelColor>::Raw>,
    BO: ByteOrder,
    RawDataSlice<'a, <Gray8 as PixelColor>::Raw, BO>:
        IntoIterator<Item = <Gray8 as PixelColor>::Raw>,
{
    fn new(
        image: &RecolouredImageRaw<'a, C, BO>,
        size: Size,
        initial_skip: usize,
        row_skip: usize,
        recolor_to: (C, C),
    ) -> Self {
        let mut iter = RawDataSlice::new(image.data).into_iter();

        if initial_skip > 0 {
            iter.nth(initial_skip - 1);
        }

        // Set `remaining_y` to `0` if `width == 0` to prevent integer underflow in `next`.
        let remaining_y = if size.width > 0 { size.height } else { 0 };

        Self {
            iter,
            remaining_x: size.width,
            width: size.width,
            remaining_y,
            row_skip,
            recolor_to,
        }
    }
}

impl<'a, C, BO> Iterator for ContiguousPixels<'a, C, BO>
where
    C: PixelColor,
    BO: ByteOrder,
    RawDataSlice<'a, <Gray8 as PixelColor>::Raw, BO>:
        IntoIterator<Item = <Gray8 as PixelColor>::Raw>,
{
    type Item = C;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining_x > 0 {
            self.remaining_x -= 1;

            self.iter.next()
        } else {
            if self.remaining_y == 0 {
                return None;
            }

            self.remaining_y -= 1;
            self.remaining_x = self.width - 1;

            self.iter.nth(self.row_skip)
        }
        .map(|c| {
            if c.into_inner() != 0 {
                self.recolor_to.1
            } else {
                self.recolor_to.0
            }
        })
    }
}
