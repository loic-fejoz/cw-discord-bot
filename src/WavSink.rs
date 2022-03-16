use std::path;

use futuresdr::anyhow::Result;
use futuresdr::runtime::AsyncKernel;
use futuresdr::runtime::Block;
use futuresdr::runtime::BlockMeta;
use futuresdr::runtime::BlockMetaBuilder;
use futuresdr::runtime::MessageIo;
use futuresdr::runtime::MessageIoBuilder;
use futuresdr::runtime::StreamIo;
use futuresdr::runtime::StreamIoBuilder;
use futuresdr::runtime::WorkIo;
use futuresdr::async_trait::async_trait;
use hound;


pub struct WavSink<T> 
where
    T: Send + 'static + hound::Sample + Copy
{
    writer: hound::WavWriter<std::io::BufWriter<std::fs::File>>,
    _type: std::marker::PhantomData<T>,
}

impl<T: Send + 'static + hound::Sample + Copy> WavSink<T> {
    pub fn   new<P: AsRef<path::Path>+ std::marker::Send + Copy>(file_name: P, spec: hound::WavSpec) -> Block {
        let writer = hound::WavWriter::create(file_name, spec).unwrap();
        Block::new_async(
            BlockMetaBuilder::new("WavSink").build(),
            StreamIoBuilder::new()
                .add_input("in", std::mem::size_of::<T>())
                .build(),
            MessageIoBuilder::new().build(),
            WavSink::<T> {
                writer: writer,
                _type: std::marker::PhantomData,
            },
        )
    }
}

#[async_trait]
impl<T: Send + 'static + hound::Sample + Copy> AsyncKernel for WavSink<T> {
    async fn work(
        &mut self,
        io: &mut WorkIo,
        sio: &mut StreamIo,
        _mio: &mut MessageIo<Self>,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        let i = sio.input(0).slice::<T>();
        let items = i.len();
        if items > 0 {
                for t in i {
                    self.writer.write_sample(*t).unwrap();
                }
        }

        if sio.input(0).finished() {
            io.finished = true;
        }

        sio.input(0).consume(items);
        Ok(())
    }

    async fn init(
        &mut self,
        _sio: &mut StreamIo,
        _mio: &mut MessageIo<Self>,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        Ok(())
    }

    async fn deinit(
        &mut self,
        _sio: &mut StreamIo,
        _mio: &mut MessageIo<Self>,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
       // self.file.as_mut().unwrap().sync_all().await.unwrap();
        //self.writer.finalize().with_context(|| format!("Failed to finalize wav file"))
        Ok(())
    }
}
