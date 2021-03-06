//#![feature(trace_macros)]
//trace_macros!(true);

#![feature(plugin)]
#![plugin(rustler_codegen)]

extern crate rustler;
use rustler::{NifTerm, NifEnv, NifError, NifResult};
use rustler::{ NifEncoder, NifDecoder };
use rustler::resource::ResourceCell;
use rustler::binary::{OwnedNifBinary, NifBinary};

mod data;
use data::{ Chunk, BlockData };

extern crate opensimplex;
use opensimplex::OsnContext;

rustler_export_nifs!("Elixir.McEx.Native.Chunk",
                     [("n_create", 0, create),
                      ("n_assemble_packet", 2, assemble_packet),
                      ("n_generate_chunk", 2, generate_chunk),
                      ("n_destroy_block", 2, destroy_block),
                      ("n_gen_chunk_raw", 1, gen_chunk_raw)],
                     Some(on_load));

fn on_load(env: &NifEnv, load_info: NifTerm) -> bool {
    resource_struct_init!(Chunk, env);
    true
}

fn create<'a>(env: &'a NifEnv, args: &Vec<NifTerm>) -> Result<NifTerm<'a>, NifError> {
    let holder = ResourceCell::new(Chunk::default());
    holder.write().unwrap().set_block(0, 100, 0, BlockData::new(1, 1));
    Ok(holder.encode(env))
}

extern crate time;

#[NifTuple] struct PacketAssemblyParams { skylight: bool, entire_chunk: bool, bitmask: u32 }
#[NifTuple] struct PacketAssemblyResponse<'a> { written_bitmask: u32, size: u32, chunk_data: NifTerm<'a> }
fn assemble_packet<'a>(env: &'a NifEnv, args: &Vec<NifTerm>) -> Result<NifTerm<'a>, NifError> {
    let holder: ResourceCell<Chunk> = try!(NifDecoder::decode(args[0]));
    let params: PacketAssemblyParams = try!(NifDecoder::decode(args[1]));

    let mut_chunk = holder.write().unwrap();
    let transmit_size = mut_chunk.get_transmit_size(params.skylight, params.entire_chunk, params.bitmask as u16);

    let mut binary = OwnedNifBinary::alloc(transmit_size).unwrap();
    let (written_bitmask, size) = mut_chunk.write_transmit_data(params.skylight, params.entire_chunk, 
                                                             params.bitmask as u16, binary.as_mut_slice());

    let binary_fin = binary.release(env);

    Ok(PacketAssemblyResponse { written_bitmask: written_bitmask as u32, size: size, chunk_data: binary_fin.get_term(env) }.encode(env))
}

extern crate voxel_worldgen;

#[NifTuple] struct ChunkPos { x: i32, z: i32 }
fn generate_chunk<'a>(env: &'a NifEnv, args: &Vec<NifTerm>) -> Result<NifTerm<'a>, NifError> {
    let start_time = time::precise_time_ns();

    let holder: ResourceCell<Chunk> = try!(NifDecoder::decode(args[0]));
    let chunk_pos: ChunkPos = try!(NifDecoder::decode(args[1]));

    let mut mut_chunk = holder.write().unwrap();

    let mut rng = voxel_worldgen::XorShiftRng::new_unseeded();
    let world_gen_state = voxel_worldgen::WorldGeneratorState::new(&mut rng);

    let block_array = voxel_worldgen::test_generate_chunk(&[chunk_pos.x, chunk_pos.z]);

    for x in 0..16 {
        for z in 0..16 {
            for y in 0..256 {
                let idx = ((x & 15) | ((z & 15) << 4) | ((y & 255) << 8)) as usize;
                mut_chunk.set_block(x, y, z, BlockData::new(block_array[idx] as u16, 0));
            }
        }
    }
    mut_chunk.clean_recalc_skylight();

    Ok(15.encode(env))
}

fn gen_chunk_raw<'a>(env: &'a NifEnv, args: &Vec<NifTerm>) -> Result<NifTerm<'a>, NifError> {
    let chunk_pos: ChunkPos = try!(NifDecoder::decode(args[0]));

    let block_array = voxel_worldgen::test_generate_chunk(&[chunk_pos.x, chunk_pos.z]);

    let mut binary = OwnedNifBinary::alloc(block_array.len()).unwrap();
    let mut binary_slice = binary.as_mut_slice();

    for idx in 0..block_array.len() {
        binary_slice[idx] = block_array[idx];
    }

    Ok(binary.release(env).get_term(env))
}

fn destroy_block<'a>(env: &'a NifEnv, args: &Vec<NifTerm>) -> Result<NifTerm<'a>, NifError> {
    let holder: ResourceCell<Chunk> = try!(NifDecoder::decode(args[0]));
    let pos: NifBlockPos = try!(NifDecoder::decode(args[1]));

    let mut mut_chunk = holder.write().unwrap();
    mut_chunk.set_block(pos.x as u16, pos.y as u16, pos.z as u16, BlockData::new(0, 0));

    Ok(15.encode(env))
}

#[NifTuple] struct NifBlockPos { x: u16, y: u16, z: u16 }
#[NifTuple] struct NifBlockData { id: u16, meta: u8 }
fn set_block<'a>(env: &'a NifEnv, args: &Vec<NifTerm>) -> NifResult<NifTerm<'a>> {
    let holder: ResourceCell<Chunk> = try!(NifDecoder::decode(args[0]));

    let pos: NifBlockPos = try!(NifDecoder::decode(args[1]));
    let data: NifBlockData = try!(NifDecoder::decode(args[2]));

    let mut mut_chunk = holder.write().unwrap();
    mut_chunk.set_block(pos.x, pos.y, pos.z, 
                        BlockData::new(data.id, data.meta));

    Ok(15.encode(env))
}

/*#[NifTuple] struct NifBlockPos { x: u32, y: u32, z: u32 }
fn set_block<'a>(env: &'a NifEnv, args: &Vec<NifTerm>) -> Result<NifTerm<'a>, NifError> {
    let holder: ResourceCell<Chunk> = try!(NifDecoder::decode(args[0], env));
    let pos: NifBlockPos = try!(NifDecoder::decode(args[1], env));
    let val: u32 = try!(NifDecoder::decode(args[2], env));

    holder.write().unwrap().set_block(pos.x, pos.y, pos.y, BlockData::new_raw(val as u16));


}*/
