#![allow(warnings, unused)]
extern crate winapi;

use client::C_BaseModelEntity::m_Glow;
use winapi::um::winuser::{GetAsyncKeyState, VK_SPACE, VK_XBUTTON1};
use winapi::um::winuser::{mouse_event, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, INPUT};
use external_csgo_cheat::memory::Memory;
use std::f32::consts::PI;
//use std::thread;
//use std::f32::consts::PI;
//use crate::gui::PracticeApp;
mod client;
mod offsets;

use crate::offsets::cs2_dumper::offsets::client_dll;

#[derive(Default, Clone, Copy)]
struct Vector3 {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Default)]
struct ViewAngles {
    pitch: f32,
    yaw: f32,
}

impl ViewAngles {
    fn lerp(&self, target: ViewAngles, factor: f32) -> ViewAngles {
        let pitch = self.pitch + (target.pitch - self.pitch) * factor;
        let yaw = self.yaw + (target.yaw - self.yaw) * factor;
        ViewAngles { pitch, yaw }
    }
}

impl Vector3 {
    fn distance(&self, other: &Vector3) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2) + (self.z - other.z).powi(2)).sqrt()
    }
}

#[derive(Default)]
struct Vector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
  }

  #[derive(Default)]
  struct Vector2 {
    pub x: f32,
    pub y: f32,
  }


type view_matrix_t = [[f32; 4]; 4];


fn calculate_view_angles(localpos: Vector3, targetpos: Vector3) -> Vector3 {
    let delta_x = targetpos.x - localpos.x;
    let delta_y = targetpos.y - localpos.y;
    let delta_z = targetpos.z - localpos.z;

    let hypotenuse = (delta_x * delta_x + delta_y * delta_y).sqrt();

    let yaw = delta_y.atan2(delta_x) * (180.0 / std::f32::consts::PI);
    let pitch = (delta_z / hypotenuse).atan() * (180.0 / std::f32::consts::PI);

    Vector3 { x: -pitch, y: yaw, z: 0.0 }
}

pub fn main() {

    let app = "cs2.exe";
    let mem = Memory::new(app);

    let base = mem.get_module_adress("client.dll");

    let localplayer: usize = mem.read(base + client_dll::dwLocalPlayerPawn);

    loop {
        let mouse4_state = unsafe { GetAsyncKeyState(VK_XBUTTON1) }; 
        let pawnhealth: i32 = mem.read(localplayer + client::C_BaseEntity::m_iHealth);
        let crosshair: usize = mem.read(localplayer + client::C_CSPlayerPawnBase::m_iIDEntIndex);
        let viewangles: Vector3 = mem.read(base + client_dll::dwViewAngles);
        let shotsfired: i32 = mem.read(localplayer + client::C_CSPlayerPawn::m_iShotsFired);
        let maxalpha: f32 = mem.read(localplayer + client::C_CSPlayerPawnBase::m_flFlashMaxAlpha);
        let lpcamera: usize = mem.read(localplayer + client::C_BasePlayerPawn::m_pCameraServices);
        let entlist: usize = mem.read(base + client_dll::dwEntityList);
        let matrix: view_matrix_t = mem.read(base + client_dll::dwViewMatrix);

        //no flash
        //println!("alpha: {}", maxalpha);
        mem.write(lpcamera + client::CCSPlayerBase_CameraServices::m_iFOV, 125);
        mem.write(localplayer + client::C_CSPlayerPawnBase::m_flFlashMaxAlpha, 0.0);
        let localpos: Vector3 = mem.read(localplayer + client::C_BasePlayerPawn::m_vOldOrigin);
        let mut closest_player_distance = f32::MAX;
        let mut closest_player_head: Option<Vector3> = None;
        let mut closest_center_distance = f32::MAX;

        for i in 1..32{
            let list_entry: usize = mem.read(entlist + (8 * (i & 0x7FFF) >> 9) + 16);
            if list_entry == 0{
                continue;
            }
            //println!("list_entry: {}", list_entry);
            let playerController: usize = mem.read(list_entry + 120 * (i & 0x1FF));
            if playerController == 0{
                continue;
            }
            //println!("playercontroller: {}", playerController);
            let playerPawn: usize = mem.read(playerController + client::CCSPlayerController::m_hPlayerPawn);
            let list_entry2: usize = mem.read(entlist + (8 * ((playerPawn & 0x7FFF) >> 9) + 16) as usize);
            if list_entry2 == 0{
                continue;
            }
            //println!("list_entry2: {}", list_entry2);
            let pCSPlayerPawn: usize = mem.read(list_entry2 + (120 * (playerPawn & 0x1FF)) as usize);
            if pCSPlayerPawn == 0{
                continue;
            }
            if pCSPlayerPawn == localplayer {
                continue;
            }

            let window_width = 1920;
            let window_height = 1080;
            let playernode: usize = mem.read(pCSPlayerPawn + client::C_BaseEntity::m_pGameSceneNode);
            let skelly: usize = mem.read(playernode + client::CSkeletonInstance::m_modelState);
            //println!("skelly: {}", skelly);

            let playerpos: Vector3 = mem.read(playernode + client::CGameSceneNode::m_vecAbsOrigin);
            let playerhead: Vector3 = Vector3 { x: playerpos.x, y: playerpos.y, z: playerpos.z};
            
            let distance = ((localpos.x - playerpos.x).powi(2) + (localpos.y - playerpos.y).powi(2) + (localpos.z - playerpos.z).powi(2)).sqrt();
        
            mem.write(pCSPlayerPawn + client::C_BaseModelEntity::m_Glow + client::CGlowProperty::m_iGlowType, 3);
            let glow_color = Vector3 { x: 0.0, y: 255.0, z: 0.0 };
            mem.write(pCSPlayerPawn + client::C_BaseModelEntity::m_Glow + client::CGlowProperty::m_fGlowColor, glow_color);
            mem.write(pCSPlayerPawn + client::C_BaseModelEntity::m_Glow + client::CGlowProperty::m_glowColorOverride, 0x888010FFu32 as i32);
            mem.write(pCSPlayerPawn + client::C_BaseModelEntity::m_Glow + client::CGlowProperty::m_bFlashing, 1);
            mem.write(pCSPlayerPawn + client::C_BaseModelEntity::m_Glow + client::CGlowProperty::m_bGlowing, 1);

            let pawnhealth: i32 = mem.read(pCSPlayerPawn + client::C_BaseEntity::m_iHealth);
            // println!("health: {}", pawnhealth);
            if pawnhealth == 0 {
                continue;
            }

            let (x_screen, y_screen, _) = wts(matrix, playerpos.x, playerpos.y, playerpos.z);

            if x_screen >= 0.0 && x_screen <= window_width as f32 && y_screen >= 0.0 && y_screen <= window_height as f32 {
                // Calculate the center distance
                let center_x = window_width as f32 / 2.0;
                let center_y = window_height as f32 / 2.0;
            
                let center_distance = ((x_screen - center_x).powi(2) + (y_screen - center_y).powi(2)).sqrt();
            
                if center_distance < closest_center_distance {
                    closest_center_distance = center_distance;
                    closest_player_head = Some(playerpos);
                }
            }
        }
        //print!("m4: {}", mouse4_state);

        if mouse4_state == -32768 {
            if let Some(player_head) = closest_player_head {
                let angles = calculate_view_angles(localpos, player_head);
                if angles.x.is_finite() && angles.y.is_finite() && angles.z.is_finite() {
                    let target_angles = ViewAngles { pitch: angles.x, yaw: angles.y };
    
                    let smooth_factor = 0.5;
                    let current_angles: ViewAngles = mem.read(base + client_dll::dwViewAngles);
                    let smoothed_angles = current_angles.lerp(target_angles, smooth_factor);
                    mem.write(base + client_dll::dwViewAngles, smoothed_angles);
                }
            }
        }
        let aimpunch: Vector3 = mem.read(localplayer + client::C_CSPlayerPawn::m_aimPunchAngle);
        let shotsfire: i32 = mem.read(localplayer + client::C_CSPlayerPawn::m_iShotsFired);
        
        //println!("Crosshair: {}", crosshair);
        let mouse4_state = unsafe { GetAsyncKeyState(VK_XBUTTON1) }; 

        if mouse4_state == -32768 {
            triggerbot(&mem, crosshair);
        }

        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

fn wts(matrix: view_matrix_t, x: f32, y: f32, z: f32) -> (f32, f32, f32) {
    let _x = matrix[0][0] * x + matrix[0][1] * y + matrix[0][2] * z + matrix[0][3];
    let _y = matrix[1][0] * x + matrix[1][1] * y + matrix[1][2] * z + matrix[1][3];

    let w = matrix[3][0] * x + matrix[3][1] * y + matrix[3][2] * z + matrix[3][3];

    let inv_w = 1.0 / w;
    let _x = _x * inv_w;
    let _y = _y * inv_w;

    let mut x_screen = 1920.0 / 2.0;
    let mut y_screen = 1080.0 / 2.0;

    x_screen += 0.5 * _x * 1920.0 + 0.5;
    y_screen += 0.5 * _y * 1080.0 + 0.5;

    (x_screen, y_screen, w)
}

pub fn triggerbot(mem: &Memory, crosshair: usize) {
    let base = mem.get_module_adress("client.dll");
    let entlist: usize = mem.read(base + client_dll::dwEntityList);

    let ententry: usize = mem.read(entlist + 0x8 * (crosshair >> 9) + 0x10);
    if ententry == 0{
        return;
    }
    let entity: usize = mem.read(ententry + 120 * (crosshair & 0x1FF));
    if entity == 0{
        return;
    }
    let ententry2: usize = mem.read(entlist + (8 * ((entity & 0x7FFF) >> 9)) + 16);
    if ententry2 == 0{
        return;
    }
    let playerPawn: usize = mem.read(ententry2 + client::CCSPlayerController::m_hPlayerPawn);
    let playerPawn: usize = mem.read(entlist + (8 * ((entity & 0x7FFF) >> 9)) + 16);
    if ententry2 == 0{
        return;
    }
    let pCSPlayerPawn: usize = mem.read(ententry2 + (120 * (playerPawn & 0x1FF)) as usize);
    //println!("{}", ententry2);
    if pCSPlayerPawn == 0{
        return;
    }
    left_mouse_down();
    std::thread::sleep(std::time::Duration::from_millis(4));
    left_mouse_up();

    let entteam: i32 = mem.read(entity + client::C_BaseEntity::m_iTeamNum);

    //println!("entteam: {}", entteam);
}

fn left_mouse_down() {
    unsafe {
        mouse_event(MOUSEEVENTF_LEFTDOWN, 0, 0, 0, 0);
    }
}

fn left_mouse_up() {
    unsafe {
        mouse_event(MOUSEEVENTF_LEFTUP, 0, 0, 0, 0);
    }
}