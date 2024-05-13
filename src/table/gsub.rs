use crate::HashSet;
use ttf_parser::Face;

pub fn load_gsub(face: &Face, indices_to_load: &mut HashSet<u16>) {
    if let Some(subtable) = face.tables().gsub {
        use ttf_parser::gsub::SubstitutionSubtable;
        for lookup in subtable.lookups {
            for table in lookup.subtables.into_iter::<SubstitutionSubtable>() {
                match table {
                    SubstitutionSubtable::Single(ss) => {
                        use ttf_parser::gsub::SingleSubstitution;
                        use ttf_parser::opentype_layout::Coverage;
                        match ss {
                            SingleSubstitution::Format1 {
                                coverage,
                                delta,
                            } => match coverage {
                                Coverage::Format1 {
                                    glyphs,
                                } => {
                                    for glyph in glyphs {
                                        indices_to_load.insert((glyph.0 as i32 + delta as i32) as u16);
                                    }
                                }
                                Coverage::Format2 {
                                    records,
                                } => {
                                    for record in records {
                                        for id in record.start.0..record.end.0 {
                                            indices_to_load.insert((id as i32 + delta as i32) as u16);
                                        }
                                    }
                                }
                            },
                            SingleSubstitution::Format2 {
                                coverage: _,
                                substitutes,
                            } => {
                                for g in substitutes {
                                    indices_to_load.insert(g.0);
                                }
                            }
                        }
                    }
                    SubstitutionSubtable::Multiple(ms) => {
                        for seq in ms.sequences {
                            for g in seq.substitutes {
                                indices_to_load.insert(g.0);
                            }
                        }
                    }
                    SubstitutionSubtable::Alternate(als) => {
                        for alt in als.alternate_sets {
                            for g in alt.alternates {
                                indices_to_load.insert(g.0);
                            }
                        }
                    }
                    SubstitutionSubtable::Ligature(ls) => ls.ligature_sets.into_iter().for_each(|ls| {
                        for l in ls {
                            indices_to_load.insert(l.glyph.0);
                        }
                    }),
                    SubstitutionSubtable::ReverseChainSingle(rcsl) => {
                        for g in rcsl.substitutes {
                            indices_to_load.insert(g.0);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
