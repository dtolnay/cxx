#include <iostream>
#include <mmscenegraph.h>

namespace mmsg = mmscenegraph;

int test_a() {
    int scene_id = 42;
    mmsg::SceneGraph *scene_graph = mmsg::scene_graph_new(scene_id);
    mmsg::scene_graph_delete(scene_graph);
    return 0;
}
