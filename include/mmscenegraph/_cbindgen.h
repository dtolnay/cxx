#pragma once

#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <new>

namespace mmscenegraph {

/// The Scene Graph.
struct SceneGraph {
  int id;
};

extern "C" {

void scene_graph_delete(SceneGraph *scene_graph_ptr);

SceneGraph *scene_graph_new(int id);

} // extern "C"

} // namespace mmscenegraph
