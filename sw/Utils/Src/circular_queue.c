
#include "circular_queue.h"

#include <string.h>

CircularBuffer cq_init(CircularQueue *q,
                    uint8_t *data_block,
                    uint16_t *data_lengths,
                    uint16_t slot_size_bytes,
                    uint8_t capacity)
{
    if (q == NULL || data_block == NULL){
        return CQ_INPUT_INVALID_INPUT;
    }
    q->data_block = data_block;
    q->data_lengths = data_lengths;
    q->slot_size_bytes = slot_size_bytes;
    q->capacity = capacity;
    q->read_index = q->write_index = q->items_stored = 0;
}

CQErrorTypes_E cq_push(CircularQueue *q, const void *data, uint16_t len){
    if (q == NULL || data == NULL){
        return CQ_INPUT_INVALID_INPUT;
    }
    if (q->data_block  == NULL){
        return CQ_ERROR_NOT_INITIALIZED;
    }
    if (q->items_stored >= q->capacity){
        return CQ_ERROR_PUSH_TO_FULL_QUEUE;
    }
    if (q->slot_size_bytes < len){
        return CQ_INPUT_INVALID_INPUT;
    }

    uint8_t *slot = q->data_block + (q->write_index * q->slot_size_bytes);
    memcpy(slot, data, len);
    if (q->data_lengths){
        q->data_lengths[q->write_index] = len;
    }

    q->write_index = (q->write_index + 1) % q->capacity;
    q->items_stored++;
    return CQ_OK;
}

CQErrorTypes_E cq_pop(CircularQueue *q, void *out, uint16_t *len_out){
    if (q == NULL || out == NULL){
        return CQ_INPUT_INVALID_INPUT;
    }
    if (q->data_block  == NULL){
        return CQ_ERROR_NOT_INITIALIZED;
    }
    if (q->items_stored == 0){
        return CQ_ERROR_POP_FROM_EMPTY_QUEUE;
    }

    uint8_t *slot = q->data_block + (q->read_index * q->slot_size_bytes);
    uint16_t len = q->lengths ? q->lengths[q->read_index] : q->slot_size_bytes;

    memcpy(out, slot, len);
    if (len_out) *len_out = len;

    q->read_index = (q->read_index + 1) % q->capacity;
    q->items_stored--;
    return CQ_OK;
}