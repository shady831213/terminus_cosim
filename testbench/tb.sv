`ifndef DLM_SIZE
`define DLM_SIZE 32'd16384
`endif

`ifndef ILM_SIZE
`define ILM_SIZE 32'd4096
`endif

`ifndef GLOBAL_BASE
`define GLOBAL_BASE 32'h80000000
`endif

`ifndef GLOBAL_SIZE
`define GLOBAL_SIZE 32'h200000
`endif

`define GLOBAL_ID 32'h1000
`define CORE0_ID 32'h0
`define CORE1_ID 32'h10
`define CORE2_ID 32'h20
`define ILM_ID 32'h0
`define DLM_ID 32'h1

module TestModule(input bit clock);
    import "DPI-C" function void mb_get_space(string ch_name, output string space_name);
    import "DPI-C" function void mb_backdoor_write_u8(string space_name, longint unsigned addr, byte unsigned data);
    import "DPI-C" function void mb_backdoor_read_u8(string space_name, longint unsigned addr, output byte unsigned data);
    import "DPI-C" function void mb_backdoor_write_u16(string space_name, longint unsigned addr, shortint unsigned data);
    import "DPI-C" function void mb_backdoor_read_u16(string space_name, longint unsigned addr, output shortint unsigned data);
    import "DPI-C" function void mb_backdoor_write_u32(string space_name, longint unsigned addr, int unsigned data);
    import "DPI-C" function void mb_backdoor_read_u32(string space_name, longint unsigned addr, output int unsigned data);
    import "DPI-C" function void mb_backdoor_write_u64(string space_name, longint unsigned addr, longint unsigned data);
    import "DPI-C" function void mb_backdoor_read_u64(string space_name, longint unsigned addr, output longint unsigned data);
    import "DPI-C" function void mb_backdoor_write_string(string space_name, longint unsigned addr, string data);
    import "DPI-C" function void mb_backdoor_read_string(string space_name, longint unsigned addr, output string data);
    import "DPI-C" function void cluster_init(int unsigned num_cores);
    import "DPI-C" function void cluster_reset_core(int unsigned hartid, longint unsigned boot_addr);
    import "DPI-C" context task mb_server_run_async();
    import "DPI-C" task cluster_run_1step();
    export "DPI-C" function mem_write_bd;
    export "DPI-C" function mem_read_bd;
    export "DPI-C" function mb_exit;
    export "DPI-C" task cluster_ext_write_u8;
    export "DPI-C" task cluster_ext_read_u8;
    export "DPI-C" task cluster_ext_write_u16;
    export "DPI-C" task cluster_ext_read_u16;
    export "DPI-C" task cluster_ext_write_u32;
    export "DPI-C" task cluster_ext_read_u32;
    export "DPI-C" task cluster_ext_write_u64;
    export "DPI-C" task cluster_ext_read_u64;
    export "DPI-C" function tb_sv_call;    
    export "DPI-C" function poll_event;

    bit [7:0] global[`GLOBAL_SIZE];
    bit [7:0] core0_ilm[`ILM_SIZE];
    bit [7:0] core0_dlm[`DLM_SIZE];
    bit [7:0] core1_ilm[`ILM_SIZE];
    bit [7:0] core1_dlm[`DLM_SIZE];    
    bit [7:0] core2_ilm[`ILM_SIZE];
    bit [7:0] core2_dlm[`DLM_SIZE];

    bit tb_clock;
   
    bit reset_n;

    bit [3:0]cnt;
    bit [31:0]timeout_cnt;


    always @(posedge clock) begin
        cnt <= cnt + 1;
        timeout_cnt <= timeout_cnt + 1;
    end

    always @(posedge clock) begin
        if (&cnt) begin
            tb_clock <= ~tb_clock;
        end
    end

    initial begin
        cluster_init(3);
        cluster_reset_core(0, 64'h80100000);
        cluster_reset_core(1, 64'h80100000);
        cluster_reset_core(2, 64'h80100000);
    end

    always @(posedge clock) begin
        if (timeout_cnt > 1000) begin
            cluster_run_1step();
        end
    end

    initial begin
        mb_server_run_async();
    end

    always@(posedge clock) begin
        if (timeout_cnt >= 1000000) begin
            $display("timeout!");
            $finish();
        end
    end

    function void test_bk_u8();
        $display("test_bk_u8:");
        $display("--------------------------");
        for (int i = 0; i < 16; i++) begin
            mb_backdoor_write_u8("core0", 64'h81000000 + {32'h0, i}, i[7:0]);
        end
        for (int i = 0; i < 20; i++) begin
            bit[7:0] data;
            mb_backdoor_read_u8("core0", 64'h81000000 + {32'h0, i}, data);
            $display("buffer output %0d = 0x%0x", i, data);
        end
        $display("--------------------------");
    endfunction

    function void test_bk_u16();
        $display("test_bk_u16:");
        $display("--------------------------");
        for (int i = 0; i < 16; i++) begin
            mb_backdoor_write_u16("core0", 64'h81000000 + {32'h0, i} * 2, i[15:0] << 8);
        end
        for (int i = 0; i < 20; i++) begin
            bit[15:0] data;
            mb_backdoor_read_u16("core0", 64'h81000000 + {32'h0, i} * 2, data);
            $display("buffer output %0d = 0x%0x", i, data);
        end
        $display("--------------------------");
    endfunction

    function void test_bk_u32();
        $display("test_bk_u32:");
        $display("--------------------------");
        for (int i = 0; i < 16; i++) begin
            mb_backdoor_write_u32("core0", 64'h81000000 + {32'h0, i} * 4, i << 24);
        end
        for (int i = 0; i < 20; i++) begin
            bit[31:0] data;
            mb_backdoor_read_u32("core0", 64'h81000000 + {32'h0, i} * 4, data);
            $display("buffer output %0d = 0x%0x", i, data);
        end
        $display("--------------------------");
    endfunction

    function void test_bk_u64();
        $display("test_bk_u32:");
        $display("--------------------------");
        for (int i = 0; i < 16; i++) begin
            mb_backdoor_write_u64("core0", 64'h81000000 + {32'h0, i} * 8, {32'h0, i} << 56);
        end
        for (int i = 0; i < 20; i++) begin
            bit[63:0] data;
            mb_backdoor_read_u64("core0", 64'h81000000 + {32'h0, i} * 8, data);
            $display("buffer output %0d = 0x%0x", i, data);
        end
        $display("--------------------------");
    endfunction

    function void test_bk_string();
        string s="";
        $display("test_bk_string:");
        $display("--------------------------");
        mb_backdoor_write_string("core0", 64'h81000000, "test_bk_string");
        mb_backdoor_read_string("core0", 64'h81000000, s);
        $display("string output %s", s);
        $display("--------------------------");
    endfunction

    function automatic void mb_exit(int unsigned code);
        test_bk_u8();
        test_bk_u16();
        test_bk_u32();
        test_bk_u64();
        test_bk_string();
        $display("exit %0d!", code);
        $finish();
    endfunction

    function automatic void mem_write_bd(int unsigned id, longint unsigned addr, byte unsigned data);
        case(id)
            `GLOBAL_ID: global[addr[31:0]-`GLOBAL_BASE] = data;
            `CORE0_ID | `ILM_ID: core0_ilm[addr[31:0]] = data;
            `CORE0_ID | `DLM_ID: core0_dlm[addr[31:0]-`ILM_SIZE] = data;
            `CORE1_ID | `ILM_ID: core1_ilm[addr[31:0]] = data;
            `CORE1_ID | `DLM_ID: core1_dlm[addr[31:0]-`ILM_SIZE] = data;
            `CORE2_ID | `ILM_ID: core2_ilm[addr[31:0]] = data;
            `CORE2_ID | `DLM_ID: core2_dlm[addr[31:0]-`ILM_SIZE] = data;
            default:;
        endcase
    endfunction

    function automatic void  mem_read_bd(int unsigned id, longint unsigned addr, output byte unsigned data);
        case(id)
            `GLOBAL_ID: data = global[addr[31:0]-`GLOBAL_BASE];
            `CORE0_ID | `ILM_ID: data = core0_ilm[addr[31:0]];
            `CORE0_ID | `DLM_ID: data = core0_dlm[addr[31:0]-`ILM_SIZE];
            `CORE1_ID | `ILM_ID: data = core1_ilm[addr[31:0]];
            `CORE1_ID | `DLM_ID: data = core1_dlm[addr[31:0]-`ILM_SIZE];
            `CORE2_ID | `ILM_ID: data = core2_ilm[addr[31:0]];
            `CORE2_ID | `DLM_ID: data = core2_dlm[addr[31:0]-`ILM_SIZE];
            default:;
        endcase
    endfunction

    function automatic void cluster_ext_write_u8(int unsigned id, longint unsigned addr, byte unsigned data);
        mem_write_bd(id, addr, data);
    endfunction

    function automatic void cluster_ext_read_u8(int unsigned id, longint unsigned addr, output byte unsigned data);
        mem_read_bd(id, addr, data);
    endfunction

    function automatic void cluster_ext_write_u16(int unsigned id, longint unsigned addr, shortint unsigned data);
        for (longint i = 0; i <2; i++) begin
            mem_write_bd(id, addr+i, data[i[31:0]*8+:8]);
        end
    endfunction

    function automatic void  cluster_ext_read_u16(int unsigned id, longint unsigned addr, output shortint unsigned data);
        for (longint i = 0; i <2; i++) begin
            mem_read_bd(id, addr+i, data[i[31:0]*8+:8]);
        end
    endfunction

    function automatic void cluster_ext_write_u32(int unsigned id, longint unsigned addr, int unsigned data);
        for (longint i = 0; i <4; i++) begin
            mem_write_bd(id, addr+i, data[i[31:0]*8+:8]);
        end
    endfunction

    function automatic void cluster_ext_read_u32(int unsigned id, longint unsigned addr, output int unsigned data);
        for (longint i = 0; i <4; i++) begin
            mem_read_bd(id, addr+i, data[i[31:0]*8+:8]);
        end
    endfunction

    function automatic void cluster_ext_write_u64(int unsigned id, longint unsigned addr, longint unsigned data);
        for (longint i = 0; i <8; i++) begin
            mem_write_bd(id, addr+i, data[i[31:0]*8+:8]);
        end
    endfunction

    function automatic void cluster_ext_read_u64(int unsigned id, longint unsigned addr, output longint unsigned data);
        for (longint i = 0; i <8; i++) begin
            mem_read_bd(id, addr+i, data[i[31:0]*8+:8]);
        end
    endfunction


    bit event_table[10];

function automatic int unsigned tb_sv_call(string ch_name, 
                                        string method, 
                                        int unsigned arg_len, 
                                        int unsigned arg0,
                                        int unsigned arg1,
                                        int unsigned arg2,
                                        int unsigned arg3,
                                        output int unsigned status);
        string space;
        mb_get_space(ch_name, space);
        if (method == "sv_display1") begin
            return sv_display1(space, arg0, arg1, status);
        end
        else if (method == "sv_display2") begin
            return sv_display2(space, arg0, status);
        end
        else begin
            return 1;
        end
    endfunction

    function int unsigned sv_display1(string space, int unsigned arg0, int unsigned arg1, output int unsigned status);
        string msg;
        if (!event_table[arg1]) begin
            status = 1;
            return 0;
        end
        mb_backdoor_read_string(space, {32'b0, arg0}, msg);
        $display("[sv_display1] %s, event %d, @%t", msg, arg1, $time);
        status = 0;
        return 0;
    endfunction

    function int unsigned sv_display2(string space, int unsigned arg0, output int unsigned status);
        string msg;
        mb_backdoor_read_string(space, {32'b0, arg0}, msg);
        $display("[sv_display2] %s, @%t", msg, $time);
        status = 0;
        return 0;
    endfunction

    always@(posedge clock) begin
        for (int i = 0; i < 10; i++) begin
            if (timeout_cnt == 5000 * i) begin
                event_table[i] <= 1;
            end
        end
    end

    function automatic int unsigned poll_event(int unsigned id);
        if (id >= 10) begin
            return 32'hffffffff;
        end
        return {31'b0, event_table[id]};
    endfunction

endmodule