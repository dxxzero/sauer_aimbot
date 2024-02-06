use std::arch::asm;
use std::{thread, time};
use std::mem::transmute;
use std::ops::Sub;
use windows::{
    core::{w, PCWSTR},
    Win32::{Foundation::HMODULE, System::LibraryLoader::GetModuleHandleW},
};

fn get_module_base_address(h_module: PCWSTR) -> isize {
    let game_handle;
    match unsafe { GetModuleHandleW(h_module) } {
        Ok(n) => game_handle = n,
        Err(_) => return 0,
    };

    return unsafe { transmute(game_handle) };
}

type Traceline = extern "cdecl" fn(player: u32, distance: &f32) -> u32;
fn my_traceline(player: u32, worldpos: *mut u32, traceline: Traceline) -> u32 {
    let hit: u32;
    let distance : f32 = 0.0;
        
    unsafe {
        asm!(
            "mov edx, edx",
            "mov ecx, ecx",
            "mov dword ptr [esp+4], edi",
            "mov dword ptr [esp], ecx",
            "call eax",
            in("edx") worldpos,
            in("ecx") player,
            in("edi") &distance,
            in("eax") traceline,
            lateout("eax") hit,
        );
    }

    return hit;
}

fn bot_main() -> () {
    let gamename = w!("sauerbraten.exe");
    const PLAYER_OFFSET: isize = 0x213EA8;
    const SHOOTING_OFFSET: u32 = 0x1E0;
    const TRACELINE_OFFSET: isize = 0x18E890;
    const WORLDPOS_OFFSET: isize = 0x2979F4;
    const Y_POS_OFFSET: u32 = 0x4;
    const Z_POS_OFFSET: u32 = 0x8;
    const ROTX_OFFSET: u32 = 0x40;
    const ROTY_OFFSET: u32 = 0x3C;
    const PLAYERLIST_OFFSET: isize = 0x29CD34;
    const PLAYERLISTSIZE_OFFSET: isize = 0x29CD3C;
    

    let module_base = get_module_base_address(gamename);
    let player = unsafe { *((module_base + PLAYER_OFFSET) as *mut u32) };
    let shoot = ( player  + SHOOTING_OFFSET) as *mut u8;
    let player_list = (module_base + PLAYERLIST_OFFSET) as *mut *mut u32;
    let player_listsize = (module_base + PLAYERLISTSIZE_OFFSET) as *mut u32;

    let traceline: Traceline = unsafe {transmute(module_base + TRACELINE_OFFSET)};
    let worldpos = (module_base + WORLDPOS_OFFSET) as *mut u32;
    
    let player_rot_x = ( player + ROTX_OFFSET) as *mut f32;
    let player_rot_y = ( player + ROTY_OFFSET) as *mut f32;
    let player_pos_x = ( player ) as *mut f32;
    let player_pos_y = ( player + Y_POS_OFFSET) as *mut f32;
    let player_pos_z = ( player + Z_POS_OFFSET) as *mut f32;

    while true {
        let tmp_listsize = unsafe { *player_listsize };
        let enemy = unsafe {*((*player_list).offset(1)) };

        let enemy_pos_x = ( enemy ) as *mut f32;
        let enemy_pos_y = ( enemy + Y_POS_OFFSET) as *mut f32;
        let enemy_pos_z = (enemy + Z_POS_OFFSET) as *mut f32;
        let player_vec = Vec3 { x: unsafe { *player_pos_x }, y: unsafe { *player_pos_y }, z: unsafe { *player_pos_z } } ;
        let enemy_vec = Vec3 { x: unsafe { *enemy_pos_x }, y: unsafe { *enemy_pos_y }, z: unsafe { *enemy_pos_z } } ;
        unsafe { *player_rot_x = libm::asinf((*enemy_pos_z - *player_pos_z) / 
                player_vec.distance(enemy_vec)) / std::f32::consts::PI * 180.0 
        }
        unsafe { *player_rot_y = -libm::atan2f(*enemy_pos_x - *player_pos_x, *enemy_pos_y - *player_pos_y) / std::f32::consts::PI * 180.0 };

        let hit = my_traceline(player, worldpos, traceline);

        if hit != 0 {
            unsafe { *shoot = 1 };
        } else {
            unsafe { *shoot = 0 };
        }
        thread::sleep(time::Duration::from_millis(50));
    }

    return;
}

trait Functions {
    fn distance(self, other: Vec3) -> f32;
    fn dot(self, other: Self) -> f32;
    fn length(self) -> f32;
}

#[derive(Clone, Copy, PartialEq)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Functions for Vec3 {
    fn distance(self, other: Vec3) -> f32 {
        return (self - other).length()
    }

    #[inline]
    fn dot(self, rhs: Self) -> f32 {
        (self.x * rhs.x) + (self.y * rhs.y) + (self.z * rhs.z)
    }

    fn length(self) -> f32 {
        return self.dot(self).sqrt()
    }
}

impl Sub<Vec3> for Vec3 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x.sub(rhs.x),
            y: self.y.sub(rhs.y),
            z: self.z.sub(rhs.z),
        }
    }
}


#[no_mangle]
#[allow(non_snake_case, unused_variables)]
extern "system" fn DllMain(h_module: HMODULE, call_reason: u32, _: *mut ()) -> bool {
    if call_reason == 1 {
        thread::spawn(|| {
            bot_main()
        });
    }

    true
}
