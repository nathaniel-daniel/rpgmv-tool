use allsorts::binary::read::ReadScope;
use allsorts::font::MatchingPresentation;
use allsorts::font_data::DynamicFontTableProvider;
use allsorts::font_data::FontData;
use allsorts::glyph_position::GlyphLayout;
use allsorts::glyph_position::TextDirection;
use allsorts::gsub::FeatureMask;
use allsorts::gsub::Features;
use allsorts::tag;
use anyhow::Context;
use std::path::Path;

#[ouroboros::self_referencing]
pub struct Font {
    buffer: Vec<u8>,

    #[borrows(mut buffer)]
    scope: ReadScope<'this>,

    #[borrows(mut scope)]
    #[not_covariant]
    font_data: FontData<'this>,

    #[borrows(mut font_data)]
    #[not_covariant]
    font: allsorts::Font<DynamicFontTableProvider<'this>>,
}

/// Load a font.
pub fn load_font(path: &Path) -> anyhow::Result<Font> {
    let buffer = std::fs::read(path)?;
    let font = FontTryBuilder {
        buffer,
        scope_builder: |buffer| Ok(ReadScope::new(buffer)),
        font_data_builder: |scope| {
            scope
                .read::<FontData<'_>>()
                .context("failed to read font data")
        },
        font_builder: |font_file| {
            let table_provider = font_file
                .table_provider(0)
                .context("unable to create table provider")?;
            allsorts::Font::new(table_provider).context("unable to load font tables")
        },
    }
    .try_build()?;

    Ok(font)
}

/// Estimate the width of some text.
pub fn get_text_width(font: &mut Font, text: &str, font_size: Option<f32>) -> anyhow::Result<f32> {
    let script = tag::LATN;
    let lang = tag::DFLT;
    let variation_tuple = None;

    let text_size = font.with_font_mut(|font| {
        let scale = font_size.map(|font_size| font_size / f32::from(font.head_table.units_per_em));

        let glyphs = font.map_glyphs(text, script, MatchingPresentation::NotRequired);

        let glyph_infos = font
            .shape(
                glyphs,
                script,
                Some(lang),
                &Features::Mask(FeatureMask::default()),
                variation_tuple,
                true,
            )
            .map_err(|(err, _)| err)
            .context("failed to shape text")?;

        let mut layout = GlyphLayout::new(font, &glyph_infos, TextDirection::LeftToRight, false);
        let positions = layout.glyph_positions()?;

        let mut text_width_unscaled: i32 = 0;
        for (_glyph, position) in glyph_infos.iter().zip(&positions) {
            text_width_unscaled += position.hori_advance;
        }
        let text_width_scaled = match scale {
            Some(scale) => (text_width_unscaled as f32) * scale,
            None => text_width_unscaled as f32,
        };
        anyhow::Ok(text_width_scaled)
    })?;

    Ok(text_size)
}
