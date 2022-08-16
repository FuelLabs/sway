# Common Collections

Sway’s standard library includes a number of very useful data structures called collections. Most other data types represent one specific value, but collections can contain multiple values. Unlike the built-in array and tuple types which are allocated on the "stack" and cannot grow in size, the data these collections point to is stored either on the "heap" or in contract "storage", which means the amount of data does not need to be known at compile time and can grow as the program runs. Each kind of collection has different capabilities and costs, and choosing an appropriate one for your current situation is a skill you’ll develop over time. In this chapter, we’ll discuss three collections that are used very often in Sway programs:

A vector on the heap allows you to store a variable number of values next to each other.

A storage vector is similar to a vector on the heap but uses persistent storage.

A storage map allows you to associate a value with a particular key.

We’ll discuss how to create and update vectors, storage vectors, and storage maps, as well as what makes each special.

- [Vectors on the Heap](./vec.md)
- [Storage Vectors](./storage_vec.md)
- [Storage Maps](./storage_map.md)
