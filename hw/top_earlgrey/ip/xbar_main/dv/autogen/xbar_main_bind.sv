// Copyright lowRISC contributors.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0
//
// xbar_main_bind module generated by `tlgen.py` tool for assertions
module xbar_main_bind;

  // Host interfaces
  bind xbar_main tlul_assert #(.EndpointType("Device")) tlul_assert_host_corei (
    .clk_i  (clk_main_i),
    .rst_ni (rst_main_ni),
    .h2d    (tl_corei_i),
    .d2h    (tl_corei_o)
  );
  bind xbar_main tlul_assert #(.EndpointType("Device")) tlul_assert_host_cored (
    .clk_i  (clk_main_i),
    .rst_ni (rst_main_ni),
    .h2d    (tl_cored_i),
    .d2h    (tl_cored_o)
  );
  bind xbar_main tlul_assert #(.EndpointType("Device")) tlul_assert_host_dm_sba (
    .clk_i  (clk_main_i),
    .rst_ni (rst_main_ni),
    .h2d    (tl_dm_sba_i),
    .d2h    (tl_dm_sba_o)
  );

  // Device interfaces
  bind xbar_main tlul_assert #(.EndpointType("Host")) tlul_assert_device_rom (
    .clk_i  (clk_main_i),
    .rst_ni (rst_main_ni),
    .h2d    (tl_rom_o),
    .d2h    (tl_rom_i)
  );
  bind xbar_main tlul_assert #(.EndpointType("Host")) tlul_assert_device_debug_mem (
    .clk_i  (clk_main_i),
    .rst_ni (rst_main_ni),
    .h2d    (tl_debug_mem_o),
    .d2h    (tl_debug_mem_i)
  );
  bind xbar_main tlul_assert #(.EndpointType("Host")) tlul_assert_device_ram_main (
    .clk_i  (clk_main_i),
    .rst_ni (rst_main_ni),
    .h2d    (tl_ram_main_o),
    .d2h    (tl_ram_main_i)
  );
  bind xbar_main tlul_assert #(.EndpointType("Host")) tlul_assert_device_eflash (
    .clk_i  (clk_main_i),
    .rst_ni (rst_main_ni),
    .h2d    (tl_eflash_o),
    .d2h    (tl_eflash_i)
  );
  bind xbar_main tlul_assert #(.EndpointType("Host")) tlul_assert_device_peri (
    .clk_i  (clk_fixed_i),
    .rst_ni (rst_fixed_ni),
    .h2d    (tl_peri_o),
    .d2h    (tl_peri_i)
  );
  bind xbar_main tlul_assert #(.EndpointType("Host")) tlul_assert_device_aon (
    .clk_i  (clk_fixed_i),
    .rst_ni (rst_fixed_ni),
    .h2d    (tl_aon_o),
    .d2h    (tl_aon_i)
  );
  bind xbar_main tlul_assert #(.EndpointType("Host")) tlul_assert_device_flash_ctrl (
    .clk_i  (clk_main_i),
    .rst_ni (rst_main_ni),
    .h2d    (tl_flash_ctrl_o),
    .d2h    (tl_flash_ctrl_i)
  );
  bind xbar_main tlul_assert #(.EndpointType("Host")) tlul_assert_device_hmac (
    .clk_i  (clk_main_i),
    .rst_ni (rst_main_ni),
    .h2d    (tl_hmac_o),
    .d2h    (tl_hmac_i)
  );
  bind xbar_main tlul_assert #(.EndpointType("Host")) tlul_assert_device_aes (
    .clk_i  (clk_main_i),
    .rst_ni (rst_main_ni),
    .h2d    (tl_aes_o),
    .d2h    (tl_aes_i)
  );
  bind xbar_main tlul_assert #(.EndpointType("Host")) tlul_assert_device_keymgr (
    .clk_i  (clk_main_i),
    .rst_ni (rst_main_ni),
    .h2d    (tl_keymgr_o),
    .d2h    (tl_keymgr_i)
  );
  bind xbar_main tlul_assert #(.EndpointType("Host")) tlul_assert_device_rv_plic (
    .clk_i  (clk_main_i),
    .rst_ni (rst_main_ni),
    .h2d    (tl_rv_plic_o),
    .d2h    (tl_rv_plic_i)
  );
  bind xbar_main tlul_assert #(.EndpointType("Host")) tlul_assert_device_alert_handler (
    .clk_i  (clk_main_i),
    .rst_ni (rst_main_ni),
    .h2d    (tl_alert_handler_o),
    .d2h    (tl_alert_handler_i)
  );
  bind xbar_main tlul_assert #(.EndpointType("Host")) tlul_assert_device_csrng (
    .clk_i  (clk_main_i),
    .rst_ni (rst_main_ni),
    .h2d    (tl_csrng_o),
    .d2h    (tl_csrng_i)
  );
  bind xbar_main tlul_assert #(.EndpointType("Host")) tlul_assert_device_entropy_src (
    .clk_i  (clk_main_i),
    .rst_ni (rst_main_ni),
    .h2d    (tl_entropy_src_o),
    .d2h    (tl_entropy_src_i)
  );
  bind xbar_main tlul_assert #(.EndpointType("Host")) tlul_assert_device_nmi_gen (
    .clk_i  (clk_main_i),
    .rst_ni (rst_main_ni),
    .h2d    (tl_nmi_gen_o),
    .d2h    (tl_nmi_gen_i)
  );

endmodule
