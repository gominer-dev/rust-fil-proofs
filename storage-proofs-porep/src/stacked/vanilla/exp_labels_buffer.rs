use anyhow::Context;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use log::*;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use storage_proofs_core::util::NODE_SIZE;

pub struct ExpLabelsBuffer {
    files: Vec<File>,
    verify: bool,
}

impl ExpLabelsBuffer {
    pub fn new(num: usize, verify: bool) -> anyhow::Result<Self> {
        // let file = std::fs::OpenOptions::new().read(true).write(true).create(true).open(name)?;
        let mut files = Vec::new();
        for _ in 0..num {
            let file = tempfile::tempfile()?;
            files.push(file);
        }
        Ok(Self { files, verify })
    }
    pub fn clear(&mut self) -> anyhow::Result<()> {
        for file in &mut self.files {
            file.seek(SeekFrom::Start(0))?;
            file.set_len(0)?;
        }
        Ok(())
    }
    pub fn flip(&mut self) -> anyhow::Result<()> {
        for file in &mut self.files {
            file.seek(SeekFrom::Start(0))?;
        }
        Ok(())
    }
    pub fn get_nth_file_mut(&mut self, n: usize) -> anyhow::Result<&mut File> {
        self.files
            .get_mut(n)
            .with_context(|| format!("Could not open {}th file", n))
            .map_err(From::from)
    }
    pub fn read_channel(&mut self, n: usize, parents_i: u32) -> anyhow::Result<[u8; NODE_SIZE]> {
        let mut data = [0u8; NODE_SIZE];
        let verify = self.verify;
        let file = self.get_nth_file_mut(n)?;
        if verify {
            let parents_i_read = file.read_u32::<BigEndian>()?;
            if parents_i_read != parents_i {
                anyhow::bail!(
                    "Data order not expected: want {}, read {}",
                    parents_i,
                    parents_i_read
                );
            }
        }
        file.read_exact(&mut data)?;
        Ok(data)
    }
    pub fn write_channel(
        &mut self,
        n: usize,
        parents_i: u32,
        val: &[u8; NODE_SIZE],
    ) -> anyhow::Result<()> {
        let verify = self.verify;
        let file = self.get_nth_file_mut(n)?;
        if verify {
            file.write_u32::<BigEndian>(parents_i)?;
        }
        file.write_all(val)?;
        Ok(())
    }
}
