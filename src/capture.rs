use anyhow::Result;
use std::{sync::mpsc::Sender, time::Instant};

use v4l::{io::traits::CaptureStream, prelude::MmapStream};

#[inline(always)]
pub fn capture(tx: Sender<Vec<u8>>, mut stream: MmapStream) -> Result<()> {
    println!("Capturing frames...");
    let start = Instant::now();

    'data_loop: while let Ok((data, _metadata)) = stream.next() {
        if !data.starts_with(&[0xFF, 0xD8]) {
            eprintln!("⚠️ Dropped: Missing JPEG SOI marker (0xFFD8).");
            continue;
        }

        let mut sof_count = 0;
        for w in data.windows(2) {
            if w == [0xFF, 0xC0] {
                sof_count += 1;
                if sof_count > 1 {
                    eprintln!("⚠️ Dropped: Multiple SOF0 markers in frame.");
                    continue 'data_loop; // or break the loop and drop frame
                }
            }
        }

        // if data.windows(2).filter(|w| *w == [0xFF, 0xC0]).count() > 1 {
        //     eprintln!("⚠️ Dropped: Multiple SOF0 markers in frame.");
        //     continue;
        // }
        // if !data.ends_with(&[0xFF, 0xD9]) {
        //     eprintln!("⚠️ Dropped: Missing JPEG EOI marker (0xFFD9).");
        // }

        if tx.send(data.to_vec()).is_err() {
            eprintln!("❌ Receiver dropped. Stopping capture.");
            break;
        }
    }

    println!("Capture stopped after {:?}", start.elapsed());
    Ok(())
}
