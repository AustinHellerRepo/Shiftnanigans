// Purpose:
//      This struct will shift forward, always checking the new element state (the cell group location) against the existing element states from previous shift indexes (previous cell groups)
//      The idea is that once a bad pair of located cell groups are found, the current shifter can be pushed forward, skipping over any need to iterate over later shift indexes
//          A bonus is to store the earlier shift index, the earlier state (location), the current shift index, and the current state (location) so that when this pair is found again the current shift index can be incremented without having to do any real calculation
//              Implementation detail: maintain a Vec<Option<BTreeSet<TElement>>> where it is initialized to None for each shift index and the BTreeSet contains the ever-growing temp collection of bad states for that specific shift index
//                  It can be set back to None as each index is incremented across (from shift index 0 to n as each shift index state is found to be valid) since there's no need to look back
//                  It is filled from a master collection per shift index and state key of vectors of BTreeSets, filled as new bad pairs are discovered.
