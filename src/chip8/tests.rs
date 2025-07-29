use super::*;
    use std::collections::HashMap;

    fn setup_chip8_and_memory() -> (Chip8, Memory) {
        let chip8 = Chip8::new();
        let mem = Memory::new();
        (chip8, mem)
    }

    #[test]
    fn test_new_chip8() {
        let chip8 = Chip8::new();
        assert_eq!(chip8.cpu.v, [0x00; NUM_REGISTERS]);
        assert_eq!(chip8.cpu.idx, 0x0000);
        assert_eq!(chip8.cpu.dt, 0);
        assert_eq!(chip8.cpu.st, 0);
        assert_eq!(chip8.cpu.pc, PROGRAM_START);
        assert_eq!(chip8.cpu.sp, 0x00);
        assert_eq!(chip8.cpu.stack, [0x0000; STACK_DEPTH]);
    }

    #[test]
    fn test_chip8_with_bindings() {
        let mut chip8 = Chip8::new();
        let mut bindings = HashMap::new();
        bindings.insert(0x2, Key::W);
        bindings.insert(0x4, Key::A);
        bindings.insert(0x6, Key::D);
        bindings.insert(0x8, Key::S);
        chip8.with_bindings(bindings.clone());
        assert_eq!(chip8.keyboard.get_bindings(), bindings);
    }

    #[test]
    fn test_chip8_insert_binding() {
        let mut chip8 = Chip8::new();
        chip8.insert_binding(0x2, Key::W);
        assert_eq!(chip8.keyboard.get_by_value(0x2), Some(&Key::W));
    }

    #[test]
    fn test_chip8_set_scale() {
        let mut chip8 = Chip8::new();
        chip8.set_scale(Scale::X2);
        assert_eq!(chip8.display.get_scale() as u16, Scale::X2 as u16);
    }

    #[test]
    fn test_chip8_run() {
        let (mut chip8, mut mem) = setup_chip8_and_memory();
        mem.write_byte(PROGRAM_START, 0x60);
        mem.write_byte(PROGRAM_START + 1, 0x00);
        mem.write_byte(PROGRAM_START + 2, 0xFF);
        let result = chip8.run(&mut mem);
        assert!(result.is_err());
        assert_eq!(chip8.cpu.v[0], 0x00);
    }

    #[test]
    fn test_chip8_update() {
        let mut chip8 = Chip8::new();
        chip8.cpu.st = 5;
        chip8.update_timers();
        assert_eq!(chip8.cpu.st, 4);
    }

    mod opcode_tests {
        use super::*;

        #[test]
        fn test_opcode() {
            let opcode = OpCode::new(0x1234);
            assert_eq!(opcode.code, 0x1234);
            assert_eq!(opcode.vx(), 0x2);
            assert_eq!(opcode.vy(), 0x3);
            assert_eq!(opcode.nibble(), 0x4);
            assert_eq!(opcode.byte(), 0x34);
            assert_eq!(opcode.addr(), 0x234);
        }

        #[test]
        fn test_chip8_execute_00e0() {
            let (mut chip8, mut mem) = setup_chip8_and_memory();
            let result = chip8.execute(0x00e0, &mut mem);
            assert!(result.is_ok());
            let cleared_display = [[false; DISPLAY_HEIGHT]; DISPLAY_WIDTH];
            assert_eq!(*chip8.display.get_grid(), cleared_display);
        }

        #[test]
        fn test_chip8_execute_00ee() {
            let (mut chip8, mut mem) = setup_chip8_and_memory();
            chip8.cpu.sp = 1;
            chip8.cpu.stack[1] = 0x0200;
            let result = chip8.execute(0x00ee, &mut mem);
            assert!(result.is_ok());
            assert_eq!(chip8.cpu.pc, 0x0200);
            assert_eq!(chip8.cpu.sp, 0);
        }

        #[test]
        fn test_chip8_execute_1nnn() {
            let (mut chip8, mut mem) = setup_chip8_and_memory();
            let result = chip8.execute(0x1234, &mut mem);
            assert!(result.is_ok());
            assert_eq!(chip8.cpu.pc, 0x0234);
        }

        #[test]
        fn test_chip8_execute_2nnn() {
            let (mut chip8, mut mem) = setup_chip8_and_memory();
            let result = chip8.execute(0x2345, &mut mem);
            assert!(result.is_ok());
            assert_eq!(chip8.cpu.pc, 0x0345);
            assert_eq!(chip8.cpu.sp, 1);
            assert_eq!(chip8.cpu.stack[1], 0x0200);
        }

        #[test]
        fn test_chip8_execute_3xkk() {
            let (mut chip8, mut mem) = setup_chip8_and_memory();
            chip8.cpu.v[0] = 0x01;
            let result = chip8.execute(0x3001, &mut mem);
            assert!(result.is_ok());
            assert_eq!(chip8.cpu.pc, PROGRAM_START + 2);
        }

        #[test]
        fn test_chip8_execute_4xkk() {
            let (mut chip8, mut mem) = setup_chip8_and_memory();
            chip8.cpu.v[0] = 0x01;
            let result = chip8.execute(0x4002, &mut mem);
            assert!(result.is_ok());
            assert_eq!(chip8.cpu.pc, PROGRAM_START + 2);
        }

        #[test]
        fn test_chip8_execute_5xy0() {
            let (mut chip8, mut mem) = setup_chip8_and_memory();
            chip8.cpu.v[0] = 0x01;
            chip8.cpu.v[1] = 0x01;
            let result = chip8.execute(0x5010, &mut mem);
            assert!(result.is_ok());
            assert_eq!(chip8.cpu.pc, PROGRAM_START + 2);
        }

        #[test]
        fn test_chip8_execute_6xkk() {
            let (mut chip8, mut mem) = setup_chip8_and_memory();
            let result = chip8.execute(0x6001, &mut mem);
            assert!(result.is_ok());
            assert_eq!(chip8.cpu.v[0], 0x01);
        }

        #[test]
        fn test_chip8_execute_7xkk() {
            let (mut chip8, mut mem) = setup_chip8_and_memory();
            let result = chip8.execute(0x7001, &mut mem);
            assert!(result.is_ok());
            assert_eq!(chip8.cpu.v[0], 0x01);
        }

        #[test]
        fn test_chip8_execute_8xy0() {
            let (mut chip8, mut mem) = setup_chip8_and_memory();
            chip8.cpu.v[0] = 0x0F;
            chip8.cpu.v[1] = 0x01;
            let result = chip8.execute(0x8010, &mut mem);
            assert!(result.is_ok());
            assert_eq!(chip8.cpu.v[0], chip8.cpu.v[1]);
        }

        #[test]
        fn test_chip8_execute_8xy1() {
            let (mut chip8, mut mem) = setup_chip8_and_memory();
            chip8.cpu.v[0] = 0x0F;
            chip8.cpu.v[1] = 0x01;
            let result = chip8.execute(0x8011, &mut mem);
            assert!(result.is_ok());
            assert_eq!(chip8.cpu.v[0], 0x0F | 0x01);
        }

        #[test]
        fn test_chip8_execute_8xy2() {
            let (mut chip8, mut mem) = setup_chip8_and_memory();
            chip8.cpu.v[0] = 0x0F;
            chip8.cpu.v[1] = 0x01;
            let result = chip8.execute(0x8012, &mut mem);
            assert!(result.is_ok());
            assert_eq!(chip8.cpu.v[0], 0x0F & 0x01);
        }

        #[test]
        fn test_chip8_execute_8xy3() {
            let (mut chip8, mut mem) = setup_chip8_and_memory();
            chip8.cpu.v[0] = 0x0F;
            chip8.cpu.v[1] = 0x01;
            let result = chip8.execute(0x8013, &mut mem);
            assert!(result.is_ok());
            assert_eq!(chip8.cpu.v[0], 0x0F ^ 0x01);
        }

        #[test]
        fn test_chip8_execute_8xy4_no_carry() {
            let (mut chip8, mut mem) = setup_chip8_and_memory();
            chip8.cpu.v[0] = 0x0F;
            chip8.cpu.v[1] = 0x01;
            let result = chip8.execute(0x8014, &mut mem);
            assert!(result.is_ok());
            assert_eq!(chip8.cpu.v[0], 0x0F + 0x01);
            assert_eq!(chip8.cpu.v[FLAG_REGISTER], 0x00);
        }

        #[test]
        fn test_chip8_execute_8xy4_carry() {
            let (mut chip8, mut mem) = setup_chip8_and_memory();
            chip8.cpu.v[0] = 0xFF;
            chip8.cpu.v[1] = 0x01;
            let result = chip8.execute(0x8014, &mut mem);
            assert!(result.is_ok());
            assert_eq!(chip8.cpu.v[0], 0xFFu8.wrapping_add(0x01));
            assert_eq!(chip8.cpu.v[FLAG_REGISTER], 0x01);
        }

        #[test]
        fn test_chip8_execute_8xy5_no_borrow() {
            let (mut chip8, mut mem) = setup_chip8_and_memory();
            chip8.cpu.v[0] = 0x0F;
            chip8.cpu.v[1] = 0x01;
            let result = chip8.execute(0x8015, &mut mem);
            assert!(result.is_ok());
            assert_eq!(chip8.cpu.v[0], 0x0F - 0x01);
            assert_eq!(chip8.cpu.v[FLAG_REGISTER], 0x01);
        }

        #[test]
        fn test_chip8_execute_8xy5_borrow() {
            let (mut chip8, mut mem) = setup_chip8_and_memory();
            chip8.cpu.v[0] = 0x00;
            chip8.cpu.v[1] = 0x01;
            let result = chip8.execute(0x8015, &mut mem);
            assert!(result.is_ok());
            assert_eq!(chip8.cpu.v[0], 0x00u8.wrapping_sub(0x01));
            assert_eq!(chip8.cpu.v[FLAG_REGISTER], 0x00);
        }

        #[test]
        fn test_chip8_execute_8xy6_even() {
            let (mut chip8, mut mem) = setup_chip8_and_memory();
            chip8.cpu.v[0] = 0x20;
            let result = chip8.execute(0x8006, &mut mem);
            assert!(result.is_ok());
            assert_eq!(chip8.cpu.v[0], 0x10);
            assert_eq!(chip8.cpu.v[FLAG_REGISTER], 0x00);
        }

        #[test]
        fn test_chip8_execute_8xy6_odd() {
            let (mut chip8, mut mem) = setup_chip8_and_memory();
            chip8.cpu.v[0] = 0x21;
            let result = chip8.execute(0x8006, &mut mem);
            assert!(result.is_ok());
            assert_eq!(chip8.cpu.v[0], 0x10);
            assert_eq!(chip8.cpu.v[FLAG_REGISTER], 0x01);
        }

        #[test]
        fn test_chip8_execute_8xy7_no_borrow() {
            let (mut chip8, mut mem) = setup_chip8_and_memory();
            chip8.cpu.v[0] = 0x01;
            chip8.cpu.v[1] = 0x0F;
            let result = chip8.execute(0x8017, &mut mem);
            assert!(result.is_ok());
            assert_eq!(chip8.cpu.v[0], 0x0F - 0x01);
            assert_eq!(chip8.cpu.v[FLAG_REGISTER], 0x01);
        }

        #[test]
        fn test_chip8_execute_8xy7_borrow() {
            let (mut chip8, mut mem) = setup_chip8_and_memory();
            chip8.cpu.v[0] = 0x0F;
            chip8.cpu.v[1] = 0x01;
            let result = chip8.execute(0x8017, &mut mem);
            assert!(result.is_ok());
            assert_eq!(chip8.cpu.v[0], 0x01u8.wrapping_sub(0x0F));
            assert_eq!(chip8.cpu.v[FLAG_REGISTER], 0x00);
        }

        #[test]
        fn test_chip8_execute_8xye_no_carry() {
            let (mut chip8, mut mem) = setup_chip8_and_memory();
            chip8.cpu.v[0] = 0x20;
            let result = chip8.execute(0x800e, &mut mem);
            assert!(result.is_ok());
            assert_eq!(chip8.cpu.v[0], 0x40);
            assert_eq!(chip8.cpu.v[FLAG_REGISTER], 0x00);
        }

        #[test]
        fn test_chip8_execute_8xye_carry() {
            let (mut chip8, mut mem) = setup_chip8_and_memory();
            chip8.cpu.v[0] = 0x81;
            let result = chip8.execute(0x800e, &mut mem);
            assert!(result.is_ok());
            assert_eq!(chip8.cpu.v[0], 0x02);
            assert_eq!(chip8.cpu.v[FLAG_REGISTER], 0x01);
        }

        #[test]
        fn test_chip8_execute_9xy0() {
            let (mut chip8, mut mem) = setup_chip8_and_memory();
            chip8.cpu.v[0] = 0x01;
            chip8.cpu.v[1] = 0x02;
            let result = chip8.execute(0x9010, &mut mem);
            assert!(result.is_ok());
            assert_eq!(chip8.cpu.pc, PROGRAM_START + 2);
        }

        #[test]
        fn test_chip8_execute_annn() {
            let (mut chip8, mut mem) = setup_chip8_and_memory();
            let result = chip8.execute(0xA123, &mut mem);
            assert!(result.is_ok());
            assert_eq!(chip8.cpu.idx, 0x0123);
        }

        #[test]
        fn test_chip8_execute_bnnn() {
            let (mut chip8, mut mem) = setup_chip8_and_memory();
            chip8.cpu.v[0] = 0x01;
            let result = chip8.execute(0xB123, &mut mem);
            assert!(result.is_ok());
            assert_eq!(chip8.cpu.pc, 0x0123 + 0x01);
        }

        #[test]
        fn test_chip8_execute_cxkk() {
            let (mut chip8, mut mem) = setup_chip8_and_memory();
            let result = chip8.execute(0xC0FF, &mut mem);
            assert!(result.is_ok());
            assert_ne!(chip8.cpu.v[0], 0x00);
        }

        #[test]
        fn test_chip8_execute_dxyn_no_collision() {
            let (mut chip8, mut mem) = setup_chip8_and_memory();
            let result = chip8.execute(0xD005, &mut mem);
            assert!(result.is_ok());
            assert_eq!(chip8.cpu.v[FLAG_REGISTER], 0x00);
        }

        #[test]
        fn test_chip8_execute_dxyn_collision() {
            let (mut chip8, mut mem) = setup_chip8_and_memory();
            chip8.display.draw(0, 0, (0..5).map(|_| 0xFF));
            let result = chip8.execute(0xD005, &mut mem);
            assert!(result.is_ok());
            assert_eq!(chip8.cpu.v[FLAG_REGISTER], 0x01);
        }

        #[test]
        fn test_chip8_execute_ex9e() {
            let (mut chip8, mut mem) = setup_chip8_and_memory();
            chip8.cpu.v[0] = 0x01;
            let _ = chip8.display.init();
            let result = chip8.execute(0xE09E, &mut mem);
            assert!(result.is_ok());
            assert_eq!(chip8.cpu.pc, PROGRAM_START);
        }

        #[test]
        fn test_chip8_execute_exa1() {
            let (mut chip8, mut mem) = setup_chip8_and_memory();
            chip8.cpu.v[0] = 0x01;
            let _ = chip8.display.init();
            let result = chip8.execute(0xE0A1, &mut mem);
            assert!(result.is_ok());
            assert_eq!(chip8.cpu.pc, PROGRAM_START + 2);
        }

        #[test]
        fn test_chip8_execute_fx07() {
            let (mut chip8, mut mem) = setup_chip8_and_memory();
            chip8.cpu.dt = 0x05;
            let result = chip8.execute(0xF007, &mut mem);
            assert!(result.is_ok());
            assert_eq!(chip8.cpu.v[0], 0x05);
        }

        #[test]
        fn test_chip8_execute_fx15() {
            let (mut chip8, mut mem) = setup_chip8_and_memory();
            chip8.cpu.v[0] = 0x05;
            let result = chip8.execute(0xF015, &mut mem);
            assert!(result.is_ok());
            assert_eq!(chip8.cpu.dt, 0x05);
        }

        #[test]
        fn test_chip8_execute_fx18() {
            let (mut chip8, mut mem) = setup_chip8_and_memory();
            chip8.cpu.v[0] = 0x05;
            let result = chip8.execute(0xF018, &mut mem);
            assert!(result.is_ok());
            assert_eq!(chip8.cpu.st, 0x05);
        }

        #[test]
        fn test_chip8_execute_fx1e() {
            let (mut chip8, mut mem) = setup_chip8_and_memory();
            chip8.cpu.idx = 0x05;
            chip8.cpu.v[0] = 0x05;
            let result = chip8.execute(0xF01E, &mut mem);
            assert!(result.is_ok());
            assert_eq!(chip8.cpu.idx, 0x0A);
        }

        #[test]
        fn test_chip8_execute_fx29() {
            let (mut chip8, mut mem) = setup_chip8_and_memory();
            chip8.cpu.v[0] = 0x05;
            let result = chip8.execute(0xF029, &mut mem);
            assert!(result.is_ok());
            assert_eq!(chip8.cpu.idx, 0x05 * SPRITE_SIZE as u16);
        }

        #[test]
        fn test_chip8_execute_fx33() {
            let (mut chip8, mut mem) = setup_chip8_and_memory();
            chip8.cpu.idx = 0x05;
            chip8.cpu.v[0] = 123;
            let result = chip8.execute(0xF033, &mut mem);
            assert!(result.is_ok());
            assert_eq!(mem.read_byte(0x05), 1);
            assert_eq!(mem.read_byte(0x06), 2);
            assert_eq!(mem.read_byte(0x07), 3);
        }

        #[test]
        fn test_chip8_execute_fx55() {
            let (mut chip8, mut mem) = setup_chip8_and_memory();
            chip8.cpu.idx = 0x05;
            chip8.cpu.v[0] = 0x01;
            chip8.cpu.v[1] = 0x02;
            let result = chip8.execute(0xF155, &mut mem);
            assert!(result.is_ok());
            assert_eq!(mem.read_byte(0x05), 0x01);
            assert_eq!(mem.read_byte(0x06), 0x02);
        }

        #[test]
        fn test_chip8_execute_fx65() {
            let (mut chip8, mut mem) = setup_chip8_and_memory();
            chip8.cpu.idx = 0x05;
            mem.write_byte(0x05, 0x01);
            mem.write_byte(0x06, 0x02);
            let result = chip8.execute(0xF165, &mut mem);
            assert!(result.is_ok());
            assert_eq!(chip8.cpu.v[0], 0x01);
            assert_eq!(chip8.cpu.v[1], 0x02);
        }
    }