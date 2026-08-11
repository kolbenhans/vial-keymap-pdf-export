// New (not from keypeek): text-width measurement so labels can be centered
// and shrunk/abbreviated to fit their keycap box, using the same TTF we
// embed into the PDF.

pub struct FontMetrics<'a> {
    face: ttf_parser::Face<'a>,
}

impl<'a> FontMetrics<'a> {
    pub fn new(font_bytes: &'a [u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let face = ttf_parser::Face::parse(font_bytes, 0)?;
        Ok(Self { face })
    }

    /// Width of `text` set at `font_size_pt`, in mm.
    pub fn width_mm(&self, text: &str, font_size_pt: f32) -> f32 {
        let units_per_em = self.face.units_per_em() as f32;
        let advance_units: u32 = text
            .chars()
            .filter_map(|c| self.face.glyph_index(c))
            .map(|gid| self.face.glyph_hor_advance(gid).unwrap_or(0) as u32)
            .sum();
        let pt = advance_units as f32 / units_per_em * font_size_pt;
        pt * 0.3527778 // pt -> mm
    }
}
