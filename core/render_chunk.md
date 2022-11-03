# Chunk render process v0.0.1

1. chunk contains voxels
2. generate voxels for a chunk
3. do not render a voxel in a chunk if it is an Air
4. when a voxel is generated, check his faces if the are required to be visible.
5. do not render not visible faces
6. generate a mesh of one type block
7. instancing is not suiteble for the moment, because it does not allow exclude not visible vertecies