use std::io::{self, Seek, SeekFrom, Write};
use nom::{
    bytes::complete::{tag, take},
    number::complete::{le_u16, le_u32, le_u8},
    sequence::tuple,
    IResult,
};

#[derive(Debug)]
pub struct SfromHeader {
    pub magic: u32, // 0x00000100
    pub file_size: u32,
    pub rom_location: u32, // Usually 0x30
    pub pcm_samples_location: u32,
    pub pcm_footer_location: u32,
    pub footer_location: u32,
    pub sdd1_data_offset: u32,
    pub reserved1: u32, // 0x00000000
    // unknown flag
    pub unknown1: u32,
    // 0x8
    pub wiiu_game_id: [u8; 8],
    pub reserved2: u32, // 0x00000000
}

#[derive(Debug)]
pub struct SfromFooter {
    pub fps: u8, // 0x3C = 60fps, 0x32 = 50fps
    pub rom_size: u32,
    pub pcm_samples_size: u32,
    pub pcm_footer_size: u32,
    pub preset_id: u16,
    pub player_count: u8,
    pub sound_volume: u8,
    pub rom_type: u8, // 0x14 = LoROM, 0x15 = HiROM
    pub enhancement_chip: u8,
    pub unknown1: u32, // Usually 0x1
    pub unknown2: u32, // Always 0x1
}

#[derive(Debug)]
pub struct GameTagData {
    /// Threshold for Armet,
    /// the Epilepsy reduction filter
    pub armet_threshold: Option<[u8; 3]>, // Tag 'A'
    pub sdd1_data: Option<Vec<u8>>,       // Tag 'D'
    pub preset_id: Option<u16>,           // Tag 'G'
    pub flags: Option<[u8; 7]>,           // Tag 'P'
    pub unknown_s: Option<[u8; 3]>,       // Tag 'S'
    pub superfx_clock: Option<u16>,       // Tag 'U'
    pub armet_version: Option<u8>,        // Tag 'a'
    pub snes_header_location: Option<u8>, // Tag 'c'
    pub unknown_d: Option<u8>,            // Tag 'd'
    pub enhancement_chip: Option<u8>,     // Tag 'e'
    pub resolution_ratio: Option<u8>,     // Tag 'h'
    pub unknown_j: Option<u8>,            // Tag 'j'
    pub mouse_flag: Option<u8>,           // Tag 'm'
    pub max_players: Option<u8>,          // Tag 'p'
    pub visible_height: Option<u8>,       // Tag 'r'
    pub unknown_t: Option<u8>,            // Tag 't'
    pub volume: Option<u8>,               // Tag 'v'
}

#[repr(u8)]
pub enum EnhancementChip {
    Normal = 0x00,
    Dsp1 = 0x02,
    Sdd1 = 0x03,
    Cx4 = 0x04,
    MegaManX = 0x05, // Copy protection fix?
    Sa1_1 = 0x06,
    Sa1_2 = 0x07,
    Sa1_3 = 0x08,
    Sa1_4 = 0x09,
    Sa1_5 = 0x0A,
    Sa1_6 = 0x0B,
    SuperFx = 0x0C,
}


impl SfromHeader {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (
            input,
            (
                magic,
                file_size,
                rom_location,
                pcm_samples_location,
                pcm_footer_location,
                footer_location,
                sdd1_data_offset,
                reserved1,
                unknown1,
                wiiu_game_id,
                reserved2,
            ),
        ) = tuple((
            le_u32,
            le_u32,
            le_u32,
            le_u32,
            le_u32,
            le_u32,
            le_u32,
            le_u32,
            le_u32,
            take(8usize),
            le_u32,
        ))(input)?;

        Ok((
            input,
            SfromHeader {
                magic,
                file_size,
                rom_location,
                pcm_samples_location,
                pcm_footer_location,
                footer_location,
                sdd1_data_offset,
                reserved1,
                unknown1,
                wiiu_game_id: wiiu_game_id.try_into().unwrap(),
                reserved2,
            },
        ))
    }
}

impl SfromFooter {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (
            input,
            (
                fps,
                rom_size,
                pcm_samples_size,
                pcm_footer_size,
                preset_id,
                player_count,
                sound_volume,
                rom_type,
                enhancement_chip,
                unknown1,
                unknown2,
            ),
        ) = tuple((
            le_u8, le_u32, le_u32, le_u32, le_u16, le_u8, le_u8, le_u8, le_u8, le_u32, le_u32,
        ))(input)?;

        Ok((
            input,
            SfromFooter {
                fps,
                rom_size,
                pcm_samples_size,
                pcm_footer_size,
                preset_id,
                player_count,
                sound_volume,
                rom_type,
                enhancement_chip,
                unknown1,
                unknown2,
            },
        ))
    }
}

impl GameTagData {
    fn parse_tag_a(input: &[u8]) -> IResult<&[u8], [u8; 3]> {
        let (input, _) = tag("A")(input)?;
        let (input, data) = take(3usize)(input)?;
        Ok((input, data.try_into().unwrap()))
    }

    fn parse_tag_d(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
        let (input, _) = tag("D")(input)?;
        let (input, size) = le_u24(input)?;
        let (input, data) = take(size as usize)(input)?;
        Ok((input, data.to_vec()))
    }

    fn parse_tag_g(input: &[u8]) -> IResult<&[u8], u16> {
        let (input, _) = tag("G")(input)?;
        let (input, _) = take(3usize)(input)?; // Skip 3 bytes
        let (input, preset_id) = le_u16(input)?;
        Ok((input, preset_id))
    }

    // Add more tag parsers as needed...

    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let mut tag_data = GameTagData {
            armet_threshold: None,
            sdd1_data: None,
            preset_id: None,
            flags: None,
            unknown_s: None,
            superfx_clock: None,
            armet_version: None,
            snes_header_location: None,
            unknown_d: None,
            enhancement_chip: None,
            resolution_ratio: None,
            unknown_j: None,
            mouse_flag: None,
            max_players: None,
            visible_height: None,
            unknown_t: None,
            volume: None,
        };

        let mut remaining = input;
        while !remaining.is_empty() {
            match remaining[0] as char {
                'A' => {
                    let (new_input, data) = Self::parse_tag_a(remaining)?;
                    tag_data.armet_threshold = Some(data);
                    remaining = new_input;
                }
                'D' => {
                    let (new_input, data) = Self::parse_tag_d(remaining)?;
                    tag_data.sdd1_data = Some(data);
                    remaining = new_input;
                }
                'G' => {
                    let (new_input, data) = Self::parse_tag_g(remaining)?;
                    tag_data.preset_id = Some(data);
                    remaining = new_input;
                }
                // Add more tag matches...
                _ => break,
            }
        }

        Ok((remaining, tag_data))
    }
}

// Helper function for 24-bit little endian integers
fn le_u24(input: &[u8]) -> IResult<&[u8], u32> {
    let (input, bytes) = take(3usize)(input)?;
    Ok((
        input,
        bytes[0] as u32 | ((bytes[1] as u32) << 8) | ((bytes[2] as u32) << 16),
    ))
}

// Example usage
pub fn parse_sfrom(input: &[u8]) -> IResult<&[u8], (SfromHeader, SfromFooter, GameTagData)> {
    let (input, header) = SfromHeader::parse(input)?;
    let (input, footer) = SfromFooter::parse(input)?;
    let (input, game_tags) = GameTagData::parse(input)?;

    Ok((input, (header, footer, game_tags)))
}

#[derive(Debug)]
pub struct Sfrom {
    pub header: SfromHeader,
    pub rom_data: Vec<u8>,
    pub pcm_data: Option<Vec<u8>>,
    pub pcm_footer: Option<Vec<u8>>,
    pub footer: SfromFooter,
    pub game_tags: GameTagData,
}

impl Sfrom {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        // First parse the header
        let (input, header) = SfromHeader::parse(input)?;

        // Extract ROM data from the specified offset to PCM samples location
        // (or footer location if no PCM data)
        let rom_end = if header.pcm_samples_location != header.footer_location {
            header.pcm_samples_location as usize
        } else {
            header.footer_location as usize
        };

        // let rom_size = rom_end - header.rom_location as usize;
        let rom_start = header.rom_location as usize;

        // Ensure we have enough data
        if input.len() < rom_end {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Eof,
            )));
        }

        // Extract ROM data
        let rom_data = input[rom_start..rom_end].to_vec();

        // Extract PCM data if present
        let (pcm_data, pcm_footer) = if header.pcm_samples_location != header.footer_location {
            let pcm_start = header.pcm_samples_location as usize;
            let pcm_end = header.pcm_footer_location as usize;
            let pcm_data = input[pcm_start..pcm_end].to_vec();

            let pcm_footer_start = header.pcm_footer_location as usize;
            let pcm_footer_end = header.footer_location as usize;
            let pcm_footer = input[pcm_footer_start..pcm_footer_end].to_vec();

            (Some(pcm_data), Some(pcm_footer))
        } else {
            (None, None)
        };

        // Move to footer position
        let footer_pos = header.footer_location as usize;
        let footer_input = &input[footer_pos..];

        // Parse footer and game tags
        let (remaining, footer) = SfromFooter::parse(footer_input)?;
        let (remaining, game_tags) = GameTagData::parse(remaining)?;

        Ok((
            remaining,
            Sfrom {
                header,
                rom_data,
                pcm_data,
                pcm_footer,
                footer,
                game_tags,
            },
        ))
    }

    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> io::Result<()> {
        // Calculate all the necessary offsets and sizes first
        let header_size = 0x30; // Standard header size
        let rom_start = header_size;
        let rom_end = rom_start + self.rom_data.len();

        let (pcm_start, pcm_footer_start, footer_start) =
            if let (Some(pcm), Some(pcm_footer)) = (&self.pcm_data, &self.pcm_footer) {
                (
                    rom_end,
                    rom_end + pcm.len(),
                    rom_end + pcm.len() + pcm_footer.len(),
                )
            } else {
                (rom_end, rom_end, rom_end)
            };

        // Calculate game tags size
        let mut game_tags_size = 0;
        if self.game_tags.armet_threshold.is_some() {
            game_tags_size += 4;
        }
        if let Some(data) = &self.game_tags.sdd1_data {
            game_tags_size += 4 + data.len();
        }
        // ... calculate sizes for other tags ...

        let total_size = footer_start + 0x23 + game_tags_size;

        // Seek to start and write header
        writer.seek(SeekFrom::Start(0))?;
        writer.write_all(&0x100u32.to_le_bytes())?; // magic
        writer.write_all(&(total_size as u32).to_le_bytes())?;
        writer.write_all(&(rom_start as u32).to_le_bytes())?;
        writer.write_all(&(pcm_start as u32).to_le_bytes())?;
        writer.write_all(&(pcm_footer_start as u32).to_le_bytes())?;
        writer.write_all(&(footer_start as u32).to_le_bytes())?;
        writer.write_all(&self.header.sdd1_data_offset.to_le_bytes())?;
        writer.write_all(&0u32.to_le_bytes())?; // reserved1
        writer.write_all(&self.header.unknown1.to_le_bytes())?;
        writer.write_all(&self.header.wiiu_game_id)?;
        writer.write_all(&0u32.to_le_bytes())?; // reserved2

        // Seek to ROM start and write ROM data
        writer.seek(SeekFrom::Start(rom_start as u64))?;
        writer.write_all(&self.rom_data)?;

        // Write PCM data if present
        if let (Some(pcm), Some(pcm_footer)) = (&self.pcm_data, &self.pcm_footer) {
            writer.seek(SeekFrom::Start(pcm_start as u64))?;
            writer.write_all(pcm)?;
            writer.seek(SeekFrom::Start(pcm_footer_start as u64))?;
            writer.write_all(pcm_footer)?;
        }

        // Seek to footer position and write footer
        writer.seek(SeekFrom::Start(footer_start as u64))?;
        writer.write_all(&[self.footer.fps])?;
        writer.write_all(&self.footer.rom_size.to_le_bytes())?;
        writer.write_all(&self.footer.pcm_samples_size.to_le_bytes())?;
        writer.write_all(&self.footer.pcm_footer_size.to_le_bytes())?;
        writer.write_all(&self.footer.preset_id.to_le_bytes())?;
        writer.write_all(&[
            self.footer.player_count,
            self.footer.sound_volume,
            self.footer.rom_type,
            self.footer.enhancement_chip,
        ])?;
        writer.write_all(&self.footer.unknown1.to_le_bytes())?;
        writer.write_all(&self.footer.unknown2.to_le_bytes())?;

        // Write game tags immediately after footer
        self.write_game_tags(writer)?;

        Ok(())
    }

    fn write_game_tags<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        // Write tags in order
        if let Some(threshold) = &self.game_tags.armet_threshold {
            writer.write_all(b"A")?;
            writer.write_all(threshold)?;
        }
        if let Some(data) = &self.game_tags.sdd1_data {
            writer.write_all(b"D")?;
            writer.write_all(&(data.len() as u32).to_le_bytes()[0..3])?;
            writer.write_all(data)?;
        }
        // ... write other tags ...

        Ok(())
    }

    pub fn save_to_file(&self, path: &str) -> io::Result<()> {
        let mut file = std::fs::File::create(path)?;

        // Pre-allocate the file size
        file.set_len(self.header.file_size as u64)?;

        self.write(&mut file)
    }
}
