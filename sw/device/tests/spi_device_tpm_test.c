// Copyright lowRISC contributors.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

#include "sw/device/lib/arch/device.h"
#include "sw/device/lib/base/mmio.h"
#include "sw/device/lib/dif/dif_base.h"
#include "sw/device/lib/dif/dif_pinmux.h"
#include "sw/device/lib/dif/dif_rv_plic.h"
#include "sw/device/lib/dif/dif_spi_device.h"
#include "sw/device/lib/runtime/hart.h"
#include "sw/device/lib/runtime/irq.h"
#include "sw/device/lib/runtime/log.h"
#include "sw/device/lib/testing/test_framework/check.h"
#include "sw/device/lib/testing/test_framework/ottf_main.h"
#include "sw/device/lib/testing/test_framework/status.h"

#include "hw/top_earlgrey/sw/autogen/top_earlgrey.h"
#include "sw/device/lib/testing/autogen/isr_testutils.h"

OTTF_DEFINE_TEST_CONFIG();

static dif_spi_device_handle_t spi_device;
static dif_pinmux_t pinmux;
static dif_rv_plic_t plic;

// Enum for TPM hardware registers
typedef enum {
  ACCESS = 0x0000,
  INT_ENABLE = 0x0008,
  INT_VECTOR = 0x000C,
  INT_STATUS = 0x0010,
  INTF_CAPABILITY = 0x0014,
  STS = 0x0018,
  DATA_FIFO = 0x0024,
  INTERFACE_ID = 0x0030,
  XDATA_FIFO = 0x0080,
  DID_VID = 0x0F00,
  RID = 0x0F04,
  TEST_DELAY = 0xFFFF, // programable delay in 1k cycle
} tpm_reg_t;

typedef struct {
  uint32_t regs[16];
  uint8_t fifo[64];
  uint32_t delay;
} tpm_state_t;

// Enum for TPM command
typedef enum {
  kTpmWriteCommand = 0x0,
  kTpmReadCommand = 0x80,
} tpm_cmd_t;

enum {
  kTpmCommandMask = 0xbf,
};


const static uint32_t kTpmAddresPrefix = 0x00D40000;
const static uint32_t kTpmAddresPrefixMask = 0xffff0000;

const static uint8_t kIterations = 100;
const static uint8_t kTpmCommandRwMask = 0x80;
const static uint8_t kTpmCommandSizeMask = 0x3f;

const static dif_spi_device_tpm_config_t tpm_config = {
    .interface = kDifSpiDeviceTpmInterfaceFifo,
    .disable_return_by_hardware = false,
    .disable_address_prefix_check = false,
    .disable_locality_check = false};

const static dif_spi_device_tpm_id_t tpm_hw_id = {
      /** The vendor ID found in the TPM_DID_VID register. */
      .vendor_id=0x1234,
      /** The device ID found in the TPM_DID_VID register. */
      .device_id=0x1234,
      /** The revision ID found in the TPM_RID register. */
      .revision=0xee};

static volatile bool header_interrupt_received = false;

static void en_plic_irqs(dif_rv_plic_t *plic) {
  // Enable functional interrupts as well as error interrupts to make sure
  // everything is behaving as expected.
  top_earlgrey_plic_irq_id_t plic_irqs[] = {
      kTopEarlgreyPlicIrqIdSpiDeviceTpmHeaderNotEmpty};

  for (uint32_t i = 0; i < ARRAYSIZE(plic_irqs); ++i) {
    CHECK_DIF_OK(dif_rv_plic_irq_set_enabled(
        plic, plic_irqs[i], kTopEarlgreyPlicTargetIbex0, kDifToggleEnabled));

    // Assign a default priority
    CHECK_DIF_OK(dif_rv_plic_irq_set_priority(plic, plic_irqs[i],
                                              kDifRvPlicMaxPriority));
  }

  // Enable the external IRQ at Ibex.
  irq_global_ctrl(true);
  irq_external_ctrl(true);
}

static void en_spi_device_irqs(dif_spi_device_t *spi_device) {
  dif_spi_device_irq_t spi_device_irqs[] = {
      kDifSpiDeviceIrqTpmHeaderNotEmpty};

  for (uint32_t i = 0; i <= ARRAYSIZE(spi_device_irqs); ++i) {
    CHECK_DIF_OK(dif_spi_device_irq_set_enabled(spi_device, spi_device_irqs[i],
                                                kDifToggleEnabled));
  }
}

void ottf_external_isr(void) {
  plic_isr_ctx_t plic_ctx = {.rv_plic = &plic,
                             .hart_id = kTopEarlgreyPlicTargetIbex0};

  // We should only be receiving the tpm header interrupt during this test.
  spi_device_isr_ctx_t spi_device_ctx = {
      .spi_device = &spi_device.dev,
      .plic_spi_device_start_irq_id =
          kTopEarlgreyPlicIrqIdSpiDeviceGenericRxFull,
      .expected_irq = kDifSpiDeviceIrqTpmHeaderNotEmpty,
      .is_only_irq = true};

  top_earlgrey_plic_peripheral_t peripheral;
  dif_spi_device_irq_t spi_device_irq;
  isr_testutils_spi_device_isr(plic_ctx, spi_device_ctx, &peripheral,
                               &spi_device_irq);

  switch (spi_device_irq) {
    case kDifSpiDeviceIrqTpmHeaderNotEmpty:
      header_interrupt_received = true;
      // Disable interrupt until work is handled.
      CHECK_DIF_OK(dif_spi_device_irq_set_enabled(
          &spi_device.dev, kDifSpiDeviceIrqTpmHeaderNotEmpty,
          kDifToggleDisabled));
      break;
    default:
      LOG_ERROR("Unexpected interrupt: %d", spi_device_irq);
      break;
  }
}

static void ack_spi_tpm_header_irq(dif_spi_device_handle_t *spi_device) {
  // Clear interrupt state and re-enable interrupt.
  header_interrupt_received = false;
  CHECK_DIF_OK(dif_spi_device_irq_acknowledge(
      &spi_device->dev, kDifSpiDeviceIrqTpmHeaderNotEmpty));
  CHECK_DIF_OK(dif_spi_device_irq_set_enabled(
      &spi_device->dev, kDifSpiDeviceIrqTpmHeaderNotEmpty, kDifToggleEnabled));
}

// This routine is needed to make sure that an interrupt does not sneak in
// and jump excution away between the boolean check and the actual invocation
// of wait_for_interrupt.
static void atomic_wait_for_interrupt(void) {
  irq_global_ctrl(false);
  if (!header_interrupt_received) {
    wait_for_interrupt();
  }
  irq_global_ctrl(true);
}

void* get_sw_register_ptr(tpm_state_t* state, uint32_t addr, uint32_t *max_size) {
    void* data = NULL;
    uint16_t offset = addr & (~kTpmAddresPrefixMask);

    switch(offset) {
      case ACCESS:
        data = &state->regs[0];
        *max_size = 4;
        break;
      case INT_ENABLE:
         data = &state->regs[1];
         *max_size = 4;
         break;
      case INT_VECTOR:
        data = &state->regs[2];
         *max_size = 4;
        break;
      case INT_STATUS:
        data = &state->regs[3];
         *max_size = 4;
        break;
      case INTF_CAPABILITY:
        data = &state->regs[4];
         *max_size = 4;
        break;
      case STS:
        data = &state->regs[5];
         *max_size = 4;
        break;
      case DATA_FIFO:
        data = &state->fifo[0];
         *max_size = 64;
        break;
      case INTERFACE_ID:
        data = &state->regs[6];
         *max_size = 4;
        break;
      case XDATA_FIFO:
        data = &state->fifo[0];
         *max_size = 64;
        break;
      case DID_VID:
        data = &state->regs[7];
         *max_size = 4;
        break;
      case RID:
        data = &state->regs[8];
         *max_size = 4;
        break;
      default:
        LOG_INFO("Unknown register:0x%X", offset);
    }
    return data;
}

void setup_return_by_hw_regs(dif_spi_device_handle_t *spi) {
    LOG_INFO("SETUP RETURN-BY-HW");

    // set localtity value
    CHECK_DIF_OK(dif_spi_device_tpm_set_access_reg(&spi_device, 0, 0));
    CHECK_DIF_OK(dif_spi_device_tpm_set_access_reg(&spi_device, 1, 1));
    CHECK_DIF_OK(dif_spi_device_tpm_set_access_reg(&spi_device, 2, 2));
    CHECK_DIF_OK(dif_spi_device_tpm_set_access_reg(&spi_device, 3, 3));
    CHECK_DIF_OK(dif_spi_device_tpm_set_access_reg(&spi_device, 4, 4));

    // set STS reg
    CHECK_DIF_OK(dif_spi_device_tpm_set_sts_reg(&spi_device, 0xa1b2c3d4));

    // set INTF_CAPABILITY
    CHECK_DIF_OK(dif_spi_device_tpm_set_intf_capability_reg(&spi_device, 0xa1b2c3d4));

    // set INT_ENABLE reg
    CHECK_DIF_OK(dif_spi_device_tpm_set_int_enable_reg(&spi_device, 0xb1b2b3b4));

    // set INT_VECTOR
    CHECK_DIF_OK(dif_spi_device_tpm_set_int_vector_reg(&spi_device, 0xa1b2c3d4));

    // set INT_STATUS
    CHECK_DIF_OK(dif_spi_device_tpm_set_int_status_reg(&spi_device, 0xa1b2c3d4));

    // set RID/DID_VID
    CHECK_DIF_OK(dif_spi_device_tpm_set_id(&spi_device, tpm_hw_id));
}

// Yes it is dumb wait code.
void programed_wait(tpm_state_t* state, uint32_t addr) {
  uint16_t offset = addr & (~kTpmAddresPrefixMask);
  if((state->delay > 0) && (offset != DATA_FIFO)) {
    for(uint32_t i = 0 ; i <= state->delay; i++) {
      // this should take aroun 20us for 24MHz clock
      volatile uint32_t t = 0;
      while(t < 24) { t++; }
    }
  }
}

void update_state(tpm_state_t* state) {
  // read magic value form data fifo
  uint32_t* pattern = (uint32_t*)&state->fifo[0];
  uint32_t* value = (uint32_t*)&state->fifo[4];
  LOG_INFO("pattern:%X value:%X", *pattern, *value);
  if(*pattern == 0x01020304) {
    state->delay = *value;
    LOG_INFO("Set dealy value: %d", state->delay);
    *value = 0;
    *pattern = 0;
  }
}

void handle_read_request(dif_spi_device_handle_t *spi, uint8_t command, uint32_t addr, tpm_state_t *state) {
    uint8_t* data = NULL;
    uint32_t len = (command & kTpmCommandSizeMask) + 1;
    uint32_t max_len = 0;

    data = get_sw_register_ptr(state, addr, &max_len);
    if ((data == NULL) || (max_len == 0)) {
      LOG_INFO("Invalid data register");
      return;
    }

    if (len <= max_len) {
      CHECK_DIF_OK(dif_spi_device_tpm_write_data(&spi_device, len, data));

      // moved to end to avoid timing issue
      LOG_INFO("Read cmd req_len:%d max_len:%d", len, max_len);
      for (int i=0; i<len; i++) {
        LOG_INFO("%d 0x%X ", i, data[i]);
      }
    } else {
      LOG_INFO("Read cmd req_len:%d max_len:%d", len, max_len);
      LOG_INFO("Read incorect len");
    }
}

void handle_write_request(dif_spi_device_handle_t *spi, uint8_t command, uint32_t addr, tpm_state_t* state) {
    uint8_t* data = NULL;
    uint32_t len = (command & kTpmCommandSizeMask) + 1;
    uint32_t max_len = 0;
    uint32_t counter = 0;

    data = get_sw_register_ptr(state, addr, &max_len);
    if ((data == NULL) || (max_len == 0)) {
      LOG_INFO("Invalid data register");
      return;
    }

    if (len <= max_len) {
      // pull for data
      dif_result_t status = kDifOutOfRange;
      while (status == kDifOutOfRange) {
        status = dif_spi_device_tpm_read_data(&spi_device, len, data);
        counter++;
      };
      CHECK_DIF_OK(status);

      // move to end to not interfere with timing
      LOG_INFO("Write cmd req_len:%d max_len:%d counter:%d", len, max_len, counter);
      for (int i=0; i<len; i++) {
        LOG_INFO("%d 0x%X ", i, data[i]);
      }
    } else {
      LOG_INFO("Read incorect len");
      return;
    }
}


bool test_main(void) {
  CHECK_DIF_OK(dif_pinmux_init(
      mmio_region_from_addr(TOP_EARLGREY_PINMUX_AON_BASE_ADDR), &pinmux));

  CHECK_DIF_OK(dif_spi_device_init_handle(
      mmio_region_from_addr(TOP_EARLGREY_SPI_DEVICE_BASE_ADDR), &spi_device));

  CHECK_DIF_OK(dif_rv_plic_init(
      mmio_region_from_addr(TOP_EARLGREY_RV_PLIC_BASE_ADDR), &plic));

  // Set IoA7 for tpm csb.
  // Longer term this needs to migrate to a top specific, platform specific
  // setting.
  CHECK_DIF_OK(dif_pinmux_input_select(
      &pinmux, kTopEarlgreyPinmuxPeripheralInSpiDeviceTpmCsb,
      kTopEarlgreyPinmuxInselIoa7));

  if (kDeviceType == kDeviceSimDV || kDeviceType == kDeviceSilicon) {
    dif_pinmux_pad_attr_t out_attr;
    dif_pinmux_pad_attr_t in_attr = {
        .slew_rate = 0,
        .drive_strength = 0,
        .flags = kDifPinmuxPadAttrPullResistorEnable |
                 kDifPinmuxPadAttrPullResistorUp};

    CHECK_DIF_OK(dif_pinmux_pad_write_attrs(&pinmux, kTopEarlgreyMuxedPadsIoa7,
                                            kDifPinmuxPadKindMio, in_attr,
                                            &out_attr));
  }


  CHECK_DIF_OK(
      dif_spi_device_tpm_configure(&spi_device, kDifToggleEnabled, tpm_config));


	// set havlue of register for CRB mode
	setup_return_by_hw_regs(&spi_device);

  // enable interrupts
  en_plic_irqs(&plic);
  en_spi_device_irqs(&spi_device.dev);

  // Sync message with testbench to begin.
  LOG_INFO("Begin TPM Test");

  tpm_state_t tpm_state;

  // set miniumum value
  tpm_state.delay = 0;

  for (uint32_t i = 0; i < kIterations; i++) {
    LOG_INFO("Iteration %d", i);

    // Wait for write interrupt.
    atomic_wait_for_interrupt();

    // read header
    uint8_t command;
    uint32_t addr;
    CHECK_DIF_OK(dif_spi_device_tpm_get_command(&spi_device, &command,
                                                &addr));
    if((addr & kTpmAddresPrefixMask) != kTpmAddresPrefix) {
      LOG_INFO("Invalid prefix");
    }

    programed_wait(&tpm_state, addr);

    if((command & kTpmCommandRwMask) == kTpmReadCommand) {
      handle_read_request(&spi_device, command, addr, &tpm_state);
    } else {
      handle_write_request(&spi_device, command, addr, &tpm_state);
    }

    // Finished processing
    ack_spi_tpm_header_irq(&spi_device);

    LOG_INFO("Ack command: 0x%X addr:0x%X", command, addr);
    update_state(&tpm_state);

  }

  return true;
}
