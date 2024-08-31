# Shiftnanigans
This crate contains useful, generic, commonly needed functionality, data structures, and algorithms for iterating sequentially forward and backward via "shifting". These "shifters" allow for controlled traversal of tree-like data structures in a manner that permits easier detection of fast-failing scenarios, rather than checking collections of items.

## Features

### Shifters
- SegmentPermutationShifter
  - Transforms unpositioned line segments into localized line segments
- IndexShifter
  - Traverses a tree-like structure of items, indexing over them
- ScalingSquareBreadthFirstSearchShifter
  - Progressively shifts over all possible depths of the internal tree of integers, only permitting reaching deeper children after first iterating over all permutations of children closer to the root in a square-like pattern
- ShiftingSquareBreadthFirstSearchShifter
  - Similar to the ScalingSquareBreadthFirstSearchShifter, but the integers are now the items at the respective index
- HyperGraphClicheShifter
  - Returns the fully connected cliche graphs as they exist in the provided hypergraph, but each stateful node is provided singularly as the shifter is traversed over

### Incrementers
- BinaryDensityIncrementer
  - Returns a collection of boolean values such that each subsequent iteration increases the total number of ones progressively starting from having zero true values to having all true values
- FixedBinaryDensityIncrementer
  - The same as the BinaryDensityIncrementer but it maintains the same density of bits as it increments to the end of the permutations
- BinaryValueIncrementer
  - Returns the binary representation of all integers from zero to the provided maximum power of two based on the provided length
- LimitedIncrementer
  - A wrapper over another incrementer, only permitting a certain number of iterations as provided to the constructor
- RoundRobinIncrementer
  - A wrapper over other incrementers that traverses around to each incrementer internally, giving each a chance to return a sequence of items
- ShifterIncrementer
  - A wrapper over a shifter that traverses iteratively in a depth-first search pattern
- ShiftingCellGroupDependencyIncrementer
  - A rather complex incrementer that compares groups of cell (pixels) to each other, disallowing overlaps (specific and general), and ensuring adjacency between non-wall cell groups and wall cell groups

### PixelBoard
- PixelBoardRandomizer
  - When provided a PixelBoard, it randomizes where the pixels (cell groups) are located while avoiding overlap and maintaining adjacency between detected cell groups

## Usage

Coming soon

## Examples

None (at the moment)
