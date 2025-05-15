/**
 * sensor_board.h
 *
 * Created on: 2025-MAY-10
 *     Author: Anni
 *
 * Prototypes for setting up and running the sensor board
 */

#ifndef FUSION_SENSOR_BOARD_H
#define FUSION_SENSOR_BOARD_H

#define TASK_100HZ_TIM_INSTANCE (TIM16)
#define TASK_800HZ_TIM_INSTANCE (TIM17)
#define ASM330_SPI_INSTANCE     (SPI1)
#define M8N_UART_INSTANCE       (USART1)

void task_100hz();
void task_800hz();

void run_sensor_board();

#endif // FUSION_SENSOR_BOARD_H
