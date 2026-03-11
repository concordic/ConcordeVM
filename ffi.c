#include <stdint.h>

#if defined(__GNUC__)
  #define EXPORT __attribute__((visibility("default")))
#else
  #define EXPORT
#endif

struct Sum {
    uint16_t n;
    uint16_t m;
    uint32_t sum;
};

EXPORT struct Sum add(uint16_t a, uint16_t b){
  struct Sum s = {a, b, (uint32_t) a + b};
  return s;
}

EXPORT uint64_t max(uint64_t a, uint64_t b) {
    return a > b ? a : b;
}
