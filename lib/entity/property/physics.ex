defmodule McEx.Entity.Property.Physics do
  use McEx.Entity.Property

  alias McEx.Util.{Vec3D, AABB}

  def vec3_mul({x0, y0, z0}, {x1, y1, z1}), do: {x0*x1, y0*y1, z0*z1}
  def vec3_mul({x0, y0, z0}, fac), do: {x0*fac, y0*fac, z0*fac}
  def vec3_sub({x0, y0, z0}, {x1, y1, z1}), do: {x0-x1, y0-y1, z0-z1}
  def vec3_add({x0, y0, z0}, {x1, y1, z1}), do: {x0+x1, y0+y1, z0+z1}

  def initial(args, state) do
    prop = %{
      gravity: args.gravity,
      drag: args.drag,
      velocity: Map.get(args, :velocity, {0, 0, 0}),
      size: Map.get(args, :size, {0.25, 0.25, 0.25}),
    }
    set_prop(state, prop)
  end

  def do_collision(aabb, {xm, ym, zm}, blocks) do
    ym = Enum.reduce(blocks, ym, fn({block_aabb, block}, movement) ->
      if block == 0 do
        movement
      else
        AABB.clamp_collide_axis(:y, block_aabb, aabb, movement)
      end
    end)
    aabb = AABB.offset(aabb, {0, ym, 0})

    xm = Enum.reduce(blocks, xm, fn({block_aabb, block}, movement) ->
      if block == 0 do
        movement
      else
        AABB.clamp_collide_axis(:x, block_aabb, aabb, movement)
      end
    end)
    aabb = AABB.offset(aabb, {xm, 0, 0})

    zm = Enum.reduce(blocks, zm, fn({block_aabb, block}, movement) ->
      if block == 0 do
        movement
      else
        AABB.clamp_collide_axis(:z, block_aabb, aabb, movement)
      end
    end)

    {xm, ym, zm}
  end

  def handle_world_event(:entity_tick, _, state) do
    prop = get_prop(state)

    velocity = prop.velocity
    velocity = velocity
    |> vec3_sub(vec3_mul(velocity, prop.drag))
    |> vec3_add({0, -prop.gravity, 0})
    movement = velocity

    position = McEx.Entity.Property.Position.get_position(state)

    entity_aabb = AABB.from_pos_size(position.pos, prop.size)
    # Box the movement occurs within
    movement_area_aabb = AABB.expand(entity_aabb, movement)

    movement_area_block_positions = movement_area_aabb
    |> AABB.blocks_in
    |> Enum.take(64)
    # Fetch all blocks within the movement area
    movement_area_blocks = movement_area_block_positions
    |> Enum.map(fn pos -> {AABB.block_aabb(pos), get_block(pos, state)} end)

    # Perform actual collision
    movement_clamped = do_collision(entity_aabb, movement, movement_area_blocks)

    new_pos = Vec3D.add(position.pos, movement_clamped)
    state = McEx.Entity.Property.Position.set_position(state, %{pos: new_pos})

    prop = %{
      prop |
      velocity: velocity,
    }
    set_prop(state, prop)
  end

  def get_block(pos, state) do
    {x, _y, z} = pos
    chunk_pos = {:chunk, round(Float.floor(x / 16)), round(Float.floor(z / 16))}
    chunk_pid = McEx.Registry.chunk_server_pid(state.world_id, chunk_pos)
    GenServer.call(chunk_pid, {:get_block, pos})
  end

end
