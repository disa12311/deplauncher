use wasm_bindgen::prelude::*;
use js_sys::*;
use web_sys::*;
use std::collections::HashMap;
use nalgebra::{Vector3, Matrix4, UnitQuaternion};
use serde::{Serialize, Deserialize};

// Web-Optimized Game Engine for Deplauncher 1.12 - Enhanced Edition (Rust)
// Specifically optimized for web browsers with modern WebGL and Canvas API

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    
    #[wasm_bindgen(js_namespace = performance)]
    fn now() -> f64;
    
    #[wasm_bindgen(js_namespace = navigator)]
    fn hardwareConcurrency() -> u32;
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// === WEB-SPECIFIC CONSTANTS ===
const MAX_ENTITIES: usize = 2000;        // Optimized for web
const MAX_PARTICLES: usize = 5000;       // WebGL friendly
const MAX_LIGHTS: usize = 25;            // WebGL shader limit
const CANVAS_WIDTH: f32 = 1920.0;
const CANVAS_HEIGHT: f32 = 1080.0;
const TARGET_FPS: f32 = 60.0;
const MAX_FRAME_TIME_MS: f32 = 16.67;    // 60 FPS budget
const QUALITY_ADJUSTMENT_COOLDOWN: u32 = 60; // Frames

// === WEB BROWSER DETECTION ===

#[derive(Debug, Clone)]
pub struct BrowserCapabilities {
    pub webgl2_available: bool,
    pub hardware_acceleration: bool,
    pub max_texture_size: u32,
    pub max_vertex_attribs: u32,
    pub instanced_rendering: bool,
    pub vertex_array_objects: bool,
    pub is_mobile: bool,
    pub cpu_cores: u32,
    pub estimated_performance_tier: u8, // 0=low, 1=medium, 2=high
}

impl Default for BrowserCapabilities {
    fn default() -> Self {
        Self {
            webgl2_available: true,
            hardware_acceleration: true,
            max_texture_size: 4096,
            max_vertex_attribs: 16,
            instanced_rendering: true,
            vertex_array_objects: true,
            is_mobile: false,
            cpu_cores: 4,
            estimated_performance_tier: 2,
        }
    }
}

// === WEB-OPTIMIZED COMPONENTS ===

#[derive(Debug, Clone, Copy)]
pub struct WebTransform {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub acceleration: Vector3<f32>,
    pub rotation: f32,        // Single Y-axis rotation for web performance
    pub scale: f32,          // Uniform scale
}

impl Default for WebTransform {
    fn default() -> Self {
        Self {
            position: Vector3::zeros(),
            velocity: Vector3::zeros(),
            acceleration: Vector3::zeros(),
            rotation: 0.0,
            scale: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WebPhysics {
    pub mass: f32,
    pub drag: f32,
    pub bounciness: f32,
    pub is_kinematic: bool,
    pub use_gravity: bool,
    pub collision_radius: f32,
}

impl Default for WebPhysics {
    fn default() -> Self {
        Self {
            mass: 1.0,
            drag: 0.02,
            bounciness: 0.5,
            is_kinematic: false,
            use_gravity: true,
            collision_radius: 16.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WebRenderer {
    pub texture_id: u32,
    pub color: [f32; 4],
    pub opacity: f32,
    pub visible: bool,
    pub render_layer: u8,
    pub blend_mode: BlendMode,
}

#[derive(Debug, Clone)]
pub enum BlendMode {
    Normal,
    Additive,
    Multiply,
    Screen,
}

impl Default for WebRenderer {
    fn default() -> Self {
        Self {
            texture_id: 0,
            color: [1.0, 1.0, 1.0, 1.0],
            opacity: 1.0,
            visible: true,
            render_layer: 0,
            blend_mode: BlendMode::Normal,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WebHealth {
    pub current: f32,
    pub max: f32,
    pub regeneration: f32,
    pub last_damage_time: f32,
}

impl Default for WebHealth {
    fn default() -> Self {
        Self {
            current: 100.0,
            max: 100.0,
            regeneration: 0.0,
            last_damage_time: 0.0,
        }
    }
}

// === WEB ENTITY SYSTEM ===

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WebEntity {
    pub id: u32,
    pub name: String,
    pub tag: String,
    pub active: bool,
    
    // Core components
    pub transform: WebTransform,
    pub physics: Option<WebPhysics>,
    pub renderer: Option<WebRenderer>,
    pub health: Option<WebHealth>,
}

impl WebEntity {
    pub fn new(id: u32, name: String, position: Vector3<f32>) -> Self {
        Self {
            id,
            name,
            tag: "Default".to_string(),
            active: true,
            transform: WebTransform {
                position,
                ..Default::default()
            },
            physics: None,
            renderer: None,
            health: None,
        }
    }
    
    pub fn add_physics(&mut self, physics: WebPhysics) -> &mut Self {
        self.physics = Some(physics);
        self
    }
    
    pub fn add_renderer(&mut self, renderer: WebRenderer) -> &mut Self {
        self.renderer = Some(renderer);
        self
    }
    
    pub fn add_health(&mut self, max_health: f32) -> &mut Self {
        self.health = Some(WebHealth {
            current: max_health,
            max: max_health,
            ..Default::default()
        });
        self
    }
    
    pub fn is_alive(&self) -> bool {
        self.active && self.health.as_ref().map_or(true, |h| h.current > 0.0)
    }
    
    pub fn update(&mut self, delta_time: f32) {
        if !self.active {
            return;
        }
        
        // Update physics
        if let Some(physics) = &self.physics {
            if !physics.is_kinematic {
                // Apply drag
                self.transform.velocity *= 1.0 - (physics.drag * delta_time);
                
                // Integration
                self.transform.velocity += self.transform.acceleration * delta_time;
                self.transform.position += self.transform.velocity * delta_time;
                
                // Reset acceleration
                self.transform.acceleration = Vector3::zeros();
            }
        }
        
        // Update rotation (simple spinning for visual effect)
        self.transform.rotation += delta_time * 45.0; // 45 degrees per second
        if self.transform.rotation > 360.0 {
            self.transform.rotation -= 360.0;
        }
        
        // Boundary wrapping for web canvas
        if self.transform.position.x < 0.0 {
            self.transform.position.x = CANVAS_WIDTH;
        } else if self.transform.position.x > CANVAS_WIDTH {
            self.transform.position.x = 0.0;
        }
        
        if self.transform.position.y < 0.0 {
            self.transform.position.y = CANVAS_HEIGHT;
        } else if self.transform.position.y > CANVAS_HEIGHT {
            self.transform.position.y = 0.0;
        }
        
        // Update health regeneration
        if let Some(health) = &mut self.health {
            if health.regeneration != 0.0 {
                health.current += health.regeneration * delta_time;
                health.current = health.current.min(health.max).max(0.0);
            }
        }
    }
}

// === WEB PARTICLE SYSTEM ===

#[derive(Debug, Clone)]
pub struct WebParticle {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub color: [f32; 4],
    pub life: f32,
    pub max_life: f32,
    pub size: f32,
    pub rotation: f32,
    pub angular_velocity: f32,
    pub active: bool,
}

#[derive(Debug)]
pub struct WebParticleSystem {
    particles: Vec<WebParticle>,
    gravity: Vector3<f32>,
    next_cleanup: u32,
}

impl WebParticleSystem {
    pub fn new() -> Self {
        Self {
            particles: Vec::with_capacity(MAX_PARTICLES),
            gravity: Vector3::new(0.0, -98.0, 0.0),
            next_cleanup: 0,
        }
    }
    
    pub fn update(&mut self, delta_time: f32) {
        // Update existing particles
        for particle in &mut self.particles {
            if !particle.active {
                continue;
            }
            
            // Apply forces
            particle.velocity += self.gravity * delta_time;
            particle.position += particle.velocity * delta_time;
            particle.rotation += particle.angular_velocity * delta_time;
            
            // Update life
            particle.life -= delta_time;
            if particle.life <= 0.0 {
                particle.active = false;
                continue;
            }
            
            // Update visual properties
            let life_ratio = particle.life / particle.max_life;
            particle.color[3] = life_ratio; // Alpha fade
            particle.size *= 0.995; // Gradual shrink
        }
        
        // Periodic cleanup (every 60 frames at 60fps = 1 second)
        self.next_cleanup = self.next_cleanup.wrapping_add(1);
        if self.next_cleanup % 60 == 0 {
            self.particles.retain(|p| p.active);
        }
    }
    
    pub fn create_explosion(&mut self, position: Vector3<f32>, intensity: f32) {
        let particle_count = (intensity * 30.0) as usize;
        let max_new_particles = (MAX_PARTICLES - self.particles.len()).min(particle_count).min(50);
        
        for _ in 0..max_new_particles {
            let angle = Math::random() * 2.0 * std::f64::consts::PI;
            let elevation = Math::random() * std::f64::consts::PI - std::f64::consts::PI * 0.5;
            let speed = 80.0 + Math::random() * 120.0;
            
            let velocity = Vector3::new(
                (angle.cos() * elevation.cos() * speed) as f32,
                (elevation.sin() * speed) as f32,
                (angle.sin() * elevation.cos() * speed) as f32,
            );
            
            let particle = WebParticle {
                position,
                velocity,
                color: [1.0, 0.7, 0.2, 1.0], // Orange fire
                life: 1.0 + Math::random() as f32,
                max_life: 2.0,
                size: 3.0 + Math::random() as f32 * 4.0,
                rotation: 0.0,
                angular_velocity: (Math::random() as f32 - 0.5) * 10.0,
                active: true,
            };
            
            self.particles.push(particle);
        }
    }
    
    pub fn particle_count(&self) -> usize {
        self.particles.iter().filter(|p| p.active).count()
    }
    
    pub fn get_render_data(&self) -> Vec<f32> {
        let active_particles: Vec<&WebParticle> = self.particles.iter().filter(|p| p.active).collect();
        let mut data = Vec::with_capacity(active_particles.len() * 8);
        
        for particle in active_particles {
            data.extend_from_slice(&[
                particle.position.x,
                particle.position.y,
                particle.position.z,
                particle.size,
                particle.color[0],
                particle.color[1],
                particle.color[2],
                particle.color[3],
            ]);
        }
        
        data
    }
}

// === WEB PERFORMANCE SYSTEM ===

#[derive(Debug)]
pub struct WebPerformanceMonitor {
    pub last_frame_time: f64,
    pub frame_times: Vec<f32>,
    pub fps_counter: u32,
    pub fps_timer: f64,
    pub current_fps: f32,
    pub average_frame_time_ms: f32,
    pub quality_level: u8,
    pub adaptive_quality: bool,
    pub quality_cooldown: u32,
    pub dropped_frames: u32,
}

impl WebPerformanceMonitor {
    pub fn new() -> Self {
        Self {
            last_frame_time: now(),
            frame_times: Vec::with_capacity(60),
            fps_counter: 0,
            fps_timer: 0.0,
            current_fps: 60.0,
            average_frame_time_ms: 16.67,
            quality_level: 2, // Start with high quality
            adaptive_quality: true,
            quality_cooldown: 0,
            dropped_frames: 0,
        }
    }
    
    pub fn update(&mut self, current_time: f64) -> f32 {
        let frame_time = ((current_time - self.last_frame_time) / 1000.0) as f32;
        self.last_frame_time = current_time;
        
        // Cap delta time
        let capped_frame_time = frame_time.min(0.033); // Max 33ms
        
        // Track frame time
        self.frame_times.push(frame_time * 1000.0); // Convert to ms
        if self.frame_times.len() > 60 {
            self.frame_times.remove(0);
        }
        
        // Calculate average frame time
        if !self.frame_times.is_empty() {
            self.average_frame_time_ms = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
        }
        
        // Update FPS
        self.fps_counter += 1;
        self.fps_timer += frame_time as f64;
        if self.fps_timer >= 1.0 {
            self.current_fps = self.fps_counter as f32 / self.fps_timer as f32;
            self.fps_counter = 0;
            self.fps_timer = 0.0;
        }
        
        // Adaptive quality adjustment
        if self.adaptive_quality && self.quality_cooldown == 0 {
            if self.average_frame_time_ms > MAX_FRAME_TIME_MS * 1.3 {
                if self.quality_level > 0 {
                    self.quality_level -= 1;
                    self.quality_cooldown = QUALITY_ADJUSTMENT_COOLDOWN;
                    console_log!("Quality reduced to {} (frame time: {:.2}ms)", 
                                self.quality_level, self.average_frame_time_ms);
                }
            } else if self.average_frame_time_ms < MAX_FRAME_TIME_MS * 0.7 {
                if self.quality_level < 2 {
                    self.quality_level += 1;
                    self.quality_cooldown = QUALITY_ADJUSTMENT_COOLDOWN * 3; // Longer cooldown for increase
                    console_log!("Quality increased to {} (frame time: {:.2}ms)", 
                                self.quality_level, self.average_frame_time_ms);
                }
            }
        }
        
        if self.quality_cooldown > 0 {
            self.quality_cooldown -= 1;
        }
        
        capped_frame_time
    }
    
    pub fn is_performance_good(&self) -> bool {
        self.current_fps >= TARGET_FPS * 0.9 && self.average_frame_time_ms <= MAX_FRAME_TIME_MS * 1.1
    }
}

// === WEB INPUT SYSTEM ===

#[derive(Debug, Default)]
pub struct WebInputSystem {
    pub keys: HashMap<u32, bool>,
    pub mouse_pos: Vector3<f32>,
    pub mouse_delta: Vector3<f32>,
    pub mouse_buttons: [bool; 3],
    pub touches: Vec<(f32, f32)>,
    pub touch_active: bool,
}

impl WebInputSystem {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn set_key(&mut self, key_code: u32, pressed: bool) {
        self.keys.insert(key_code, pressed);
    }
    
    pub fn is_key_pressed(&self, key_code: u32) -> bool {
        *self.keys.get(&key_code).unwrap_or(&false)
    }
    
    pub fn set_mouse(&mut self, x: f32, y: f32, delta_x: f32, delta_y: f32) {
        self.mouse_pos = Vector3::new(x, y, 0.0);
        self.mouse_delta = Vector3::new(delta_x, delta_y, 0.0);
    }
    
    pub fn set_touch(&mut self, touches: Vec<(f32, f32)>) {
        self.touches = touches;
        self.touch_active = !self.touches.is_empty();
    }
    
    pub fn get_movement_input(&self) -> Vector3<f32> {
        let mut movement = Vector3::zeros();
        
        // WASD movement
        if self.is_key_pressed(87) { movement.y -= 1.0; } // W
        if self.is_key_pressed(83) { movement.y += 1.0; } // S
        if self.is_key_pressed(65) { movement.x -= 1.0; } // A
        if self.is_key_pressed(68) { movement.x += 1.0; } // D
        
        // Arrow keys
        if self.is_key_pressed(38) { movement.y -= 1.0; } // Up
        if self.is_key_pressed(40) { movement.y += 1.0; } // Down
        if self.is_key_pressed(37) { movement.x -= 1.0; } // Left
        if self.is_key_pressed(39) { movement.x += 1.0; } // Right
        
        // Normalize diagonal movement
        if movement.magnitude() > 1.0 {
            movement = movement.normalize();
        }
        
        movement
    }
}

// ===
