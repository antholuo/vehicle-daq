/**
 *  Auther: Hardy Yu
 *  Created on 2025-05-21
 *  Common use circular queue library
 */

#ifndef CIRCULAR_QUEUE_H_
#define CIRCULAR_QUEUE_H_

#include <stdbool.h>
#include <stdint.h>

typedef enum {
    CQ_OK = 0,
    CQ_ERROR_PUSH_TO_FULL_QUEUE = 1,
    CQ_ERROR_POP_FROM_EMPTY_QUEUE = 2,
    CQ_ERROR_NOT_INITIALIZED = 3,
    CQ_INPUT_INVALID_INPUT = 4,
} CQErrorTypes_E;

typedef struct {
    uint8_t *data_block;       // Pointer to flat memory block storing all items
    uint16_t *data_lengths;    // Optional: actual length of each item (for variable-length data like serialized packets)

    uint16_t slot_size_bytes;  // Size of each slot (maximum space reserved per item)
    uint8_t capacity;          // Maximum number of items the buffer can hold

    uint8_t read_index;        // Index of the next item to read (head)
    uint8_t write_index;       // Index of the next slot to write (tail)
    uint8_t items_stored;      // Current number of items in the buffer
} CircularQueue;

/**
 *  @brief circular queue init function
 *  @param q pointer to the uninitialized circular queue item
 *  @param data_block  pointer to the array that stores all the data
 *  @param data_lengths pointer to the array with "capacity" size,
 *         lengths of variable-length data, pass NULL for fixed-size data
 *  @param slot_size_bytes maximum length of each slot; for fixed-size data, this is sizeof(your_data_type)
 *  @param capacity maximum number of items the buffer can hold
 *  @retval error message
 */
CQErrorTypes_E cq_init(CircularQueue *q,
                    uint8_t *data_block,
                    uint16_t *data_lengths,
                    uint16_t slot_size_bytes,
                    uint8_t capacity);

CQErrorTypes_E cq_push(CircularQueue *q, const void *data, uint16_t len);

/* Note: You can choose to pass NULL to len_out if you don't wanna know (size is fixed) */
CQErrorTypes_E cq_pop(CircularQueue *q, void *out, uint16_t *len_out);

#endif // CIRCULAR_QUEUE_H_