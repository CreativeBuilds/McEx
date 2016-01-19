
defmodule McEx.Chunk.ChunkSupervisor do
  use Supervisor

  def start_link do
    Supervisor.start_link(__MODULE__, :ok, [name: McEx.Chunk.Supervisor])
  end

  def start_chunk(pos) do
    Supervisor.start_child(McEx.Chunk.Supervisor, [pos])
  end

  def init(:ok) do
    children = [
      worker(McEx.Chunk, [], restart: :transient)
    ]

    opts = [strategy: :simple_one_for_one]
    supervise(children, opts)
  end
end

defmodule McEx.Chunk do
  use GenServer
  alias McEx.Net.Connection.Write
  alias McEx.Net.Packets.Server

  use Bitwise

  def start_link(pos, opts \\ []) do
    GenServer.start_link(__MODULE__, {pos}, opts)
  end

  def send_chunk(server, writer) do
    GenServer.cast(server, {:send_chunk, writer})
  end
  def stop_chunk(server) do
    GenServer.cast(server, :stop_chunk)
  end

  def init({pos}) do
    chunk = McEx.Native.Chunk.create
    {:chunk, x, z} = pos
    McEx.Native.Chunk.generate_chunk(chunk, {x, z})
    {:ok, %{
        chunk_resource: chunk,
        pos: pos}}
  end

  def write_empty_row(bin) do
    <<bin::binary, 0::256*16>>
  end
  def write_row_data(bin, row) do
    <<bin::binary, row::binary>>
  end

  def write_chunk_packet(state) do
    alias McEx.DataTypes.Encode

    {written_mask, size, data} = McEx.Native.Chunk.assemble_packet(state.chunk_resource, {true, true, 0})

    {:chunk, x, z} = state.pos
    %McEx.Net.Packets.Server.Play.ChunkData{
      chunk_x: x, 
      chunk_z: z, 
      continuous: true, 
      section_mask: written_mask, 
      chunk_data: Encode.varint(size) <> data}
  end

  def handle_cast({:send_chunk, writer}, state) do
    Write.write_packet(writer, write_chunk_packet(state))
    {:noreply, state}
  end
  def handle_cast(:stop_chunk, state) do
    {:stop, :normal, state}
  end
end
