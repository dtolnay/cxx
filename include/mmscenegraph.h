#ifndef MMSCENEGRAPH_H
#define MMSCENEGRAPH_H

// On MSVC, <memory> will bring in the TEMPLATE macros needed
#include <memory> 

// Create create a 'make_unique' function because it's missing in older
// versions of GCC and MSVC.
//
// https://herbsutter.com/gotw/_102/
#ifdef __GNUC__
#if __cplusplus < 201402L  // Less than C++14

namespace std {
    template<typename T, typename... Args>
    std::unique_ptr<T> make_unique(Args&&... args)
    {
        return std::unique_ptr<T>(new T(std::forward<Args>(args)...));
    }
} // std namespace

#endif  // GCC_VERSION
#endif  // __GNUC__

#include <cxx.h>
#include <mmscenegraph/_cbindgen.h>
#include <mmscenegraph/_cxxbridge.h>
#include <mmscenegraph/_cpp.h>


#endif // MMSCENEGRAPH_H
