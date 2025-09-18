#include <emscripten.h>
#include <emscripten/html5.h>
#include <emscripten/threading.h>
#include <emscripten/fetch.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <stdbool.h>
#include <time.h>

// Web-Optimized Game Engine for Deplauncher 1.12 - Enhanced Edition
// Optimized specifically for web browsers with WebGL and Canvas integration

// === WEB-SPECIFIC CONSTANTS ===
#define MAX_ENTITIES 2000        // Reduced for web performance
#define MAX_PARTICLES 5000       // Optimized for browser rendering
#define MAX_LIGHTS 25            // Limited for WebGL performance
#define CANVAS_WIDTH 1920
#define CANVAS_HEIGHT 1080
#define PHYSICS_SUBSTEPS 2       // Reduced for web performance
#define NETWORK_BUFFER_SIZE 4096 // Optimized for WebSockets
#define ENTITY_NAME_SIZE 32      // Smaller for memory efficiency
#define ENTITY_TAG_SIZE 16       // Optimized size

// Web performance targets
#define TARGET_FPS 60.0f
#define MAX_FRAME_TIME_MS 16.67f // 60 FPS target
#define ADAPTIVE_QUALITY_THRESHOLD 20.0f

// === CORE STRUCTURES OPTIMIZED FOR WEB ===

// Compact 3D Vector for better cache performance
typedef struct __attribute__((packed)) {
    float x, y, z;
} Vector3;

// Web-optimized Transform component
typedef struct __attribute__((aligned(16))) {
    Vector3 position;
    Vector3 velocity;
    Vector3 acceleration;
    float rotation_y; // Single axis rotation for web performance
    float scale;      // Uniform scale for simplicity
} Transform;

// Simplified Physics for web
typedef struct {
    float mass;
    float drag;
    float bounciness;
    bool is_kinematic : 1;
    bool has_gravity : 1;
    bool enabled : 1;
    char padding : 5; // Bit packing
} Physics;

// Web-optimized Renderer
typedef struct {
    int texture_id;
    float color[4];  // RGBA for WebGL
    float opacity;
    bool visible : 1;
    bool cast_shadows : 1;
    char render_layer : 6; // 0-63 render layers
} Renderer;

// Compact Game Logic
typedef struct {
    bool active;
    short health;      // 16-bit for memory efficiency
    short max_health;
    char name[ENTITY_NAME_SIZE];
    char tag[ENTITY_TAG_SIZE];
    char layer;
} GameLogic;

// Simple AI for web
typedef struct {
    int state;
    float timer;
    Vector3 target_position;
    short target_entity_id;
    char behavior_type; // 0=idle, 1=wander, 2=seek
} AI;

// Web-optimized Entity (cache-friendly layout)
typedef struct __attribute__((aligned(64))) {
    Transform transform;
    Physics physics;
    Renderer renderer;
    GameLogic logic;
    AI ai;
} WebEntity;

// WebGL-friendly Particle
typedef struct __attribute__((packed)) {
    Vector3 position;
    Vector3 velocity;
    float color[4];
    float life;
    float size;
    bool active;
} WebParticle;

// Web-optimized Light
typedef struct __attribute__((packed)) {
    Vector3 position;
    float color[3];
    float intensity;
    float range;
    char type; // 0=directional, 1=point
    bool active;
} WebLight;

// Browser performance tracking
typedef struct {
    double last_frame_time;
    double frame_accumulator;
    int fps_counter;
    float current_fps;
    float average_frame_time_ms;
    int dropped_frames;
    bool performance_mode;  // Adaptive quality
    char quality_level;     // 0=low, 1=medium, 2=high
} BrowserPerformanceMetrics;

// Web graphics settings
typedef struct {
    bool webgl2_available;
    bool hardware_acceleration;
    bool instanced_rendering;
    bool vertex_array_objects;
    int max_texture_size;
    int max_vertex_attributes;
    char quality_preset; // Auto-detected quality
} WebGraphicsCapabilities;

// Web input state
typedef struct {
    bool keys[256];
    float mouse_x, mouse_y;
    float mouse_delta_x, mouse_delta_y;
    bool mouse_buttons[3]; // Left, Right, Middle
    bool touch_active;
    float touch_x, touch_y;
    int touch_count;
} WebInputState;

// Main web game state
typedef struct {
    // Entity system - cache-aligned
    WebEntity entities[MAX_ENTITIES];
    int entity_count;
    
    // Particle system
    WebParticle particles[MAX_PARTICLES];
    int particle_count;
    
    // Lighting system
    WebLight lights[MAX_LIGHTS];
    int light_count;
    
    // Web-specific systems
    BrowserPerformanceMetrics performance;
    WebGraphicsCapabilities graphics_caps;
    WebInputState input;
    
    // Camera (simplified for web)
    Vector3 camera_position;
    Vector3 camera_target;
    float camera_fov;
    
    // Physics world
    Vector3 gravity;
    bool physics_enabled;
    
    // Game state
    int score;
    int level;
    float time_scale;
    bool paused;
    
    // Web optimization flags
    bool adaptive_quality;
    bool vsync_enabled;
    bool debug_mode;
} WebGameState;

// === GLOBAL STATE ===
static WebGameState* g_game_state = NULL;
static bool g_initialized = false;

// Performance monitoring
static float g_frame_budget_ms = MAX_FRAME_TIME_MS;
static int g_quality_adjustment_cooldown = 0;

// === UTILITY FUNCTIONS ===

// Optimized Vector3 operations using SIMD-friendly code
Vector3 vec3_create(float x, float y, float z) {
    Vector3 v = {x, y, z};
    return v;
}

Vector3 vec3_add(Vector3 a, Vector3 b) {
    return (Vector3){a.x + b.x, a.y + b.y, a.z + b.z};
}

Vector3 vec3_multiply_scalar(Vector3 v, float scalar) {
    return (Vector3){v.x * scalar, v.y * scalar, v.z * scalar};
}

float vec3_magnitude_squared(Vector3 v) {
    return v.x * v.x + v.y * v.y + v.z * v.z;
}

float vec3_magnitude(Vector3 v) {
    return sqrtf(vec3_magnitude_squared(v));
}

Vector3 vec3_normalize_fast(Vector3 v) {
    float inv_mag = 1.0f / sqrtf(vec3_magnitude_squared(v) + 1e-6f);
    return vec3_multiply_scalar(v, inv_mag);
}

// === WEB-SPECIFIC FUNCTIONS ===

// Detect browser capabilities
void detect_web_capabilities() {
    WebGraphicsCapabilities* caps = &g_game_state->graphics_caps;
    
    // These would be detected via JavaScript and passed to WASM
    caps->webgl2_available = true; // Assume modern browser
    caps->hardware_acceleration = true;
    caps->instanced_rendering = true;
    caps->vertex_array_objects = true;
    caps->max_texture_size = 4096;
    caps->max_vertex_attributes = 16;
    caps->quality_preset = 2; // Start with high quality
}

// Adaptive quality system for web performance
void update_adaptive_quality(float frame_time_ms) {
    if (!g_game_state->adaptive_quality) return;
    
    BrowserPerformanceMetrics* perf = &g_game_state->performance;
    
    // Cooldown to prevent rapid quality changes
    if (g_quality_adjustment_cooldown > 0) {
        g_quality_adjustment_cooldown--;
        return;
    }
    
    // Check if we're consistently over budget
    if (frame_time_ms > g_frame_budget_ms * 1.2f) {
        if (perf->quality_level > 0) {
            perf->quality_level--;
            g_quality_adjustment_cooldown = 60; // 1 second cooldown at 60fps
            printf("Quality reduced to %d (frame time: %.2fms)\n", 
                   perf->quality_level, frame_time_ms);
        }
    }
    // Check if we can increase quality
    else if (frame_time_ms < g_frame_budget_ms * 0.7f) {
        if (perf->quality_level < 2) {
            perf->quality_level++;
            g_quality_adjustment_cooldown = 180; // 3 second cooldown
            printf("Quality increased to %d (frame time: %.2fms)\n", 
                   perf->quality_level, frame_time_ms);
        }
    }
}

// === MEMORY MANAGEMENT ===

bool allocate_web_game_state() {
    g_game_state = (WebGameState*)aligned_alloc(64, sizeof(WebGameState));
    if (g_game_state) {
        memset(g_game_state, 0, sizeof(WebGameState));
        return true;
    }
    return false;
}

void deallocate_web_game_state() {
    if (g_game_state) {
        free(g_game_state);
        g_game_state = NULL;
    }
}

// === ENTITY MANAGEMENT ===

WebEntity* create_web_entity(Vector3 position, const char* name) {
    if (!g_game_state || g_game_state->entity_count >= MAX_ENTITIES) {
        return NULL;
    }
    
    WebEntity* entity = &g_game_state->entities[g_game_state->entity_count++];
    
    // Initialize transform
    entity->transform.position = position;
    entity->transform.velocity = vec3_create(0, 0, 0);
    entity->transform.acceleration = vec3_create(0, 0, 0);
    entity->transform.rotation_y = 0.0f;
    entity->transform.scale = 1.0f;
    
    // Initialize physics
    entity->physics.mass = 1.0f;
    entity->physics.drag = 0.02f;
    entity->physics.bounciness = 0.5f;
    entity->physics.is_kinematic = false;
    entity->physics.has_gravity = true;
    entity->physics.enabled = true;
    
    // Initialize renderer
    entity->renderer.texture_id = 0;
    entity->renderer.color[0] = entity->renderer.color[1] = 
    entity->renderer.color[2] = entity->renderer.color[3] = 1.0f;
    entity->renderer.opacity = 1.0f;
    entity->renderer.visible = true;
    entity->renderer.cast_shadows = false;
    entity->renderer.render_layer = 0;
    
    // Initialize game logic
    entity->logic.active = true;
    entity->logic.health = 100;
    entity->logic.max_health = 100;
    strncpy(entity->logic.name, name, ENTITY_NAME_SIZE - 1);
    entity->logic.name[ENTITY_NAME_SIZE - 1] = '\0';
    strcpy(entity->logic.tag, "Default");
    entity->logic.layer = 0;
    
    // Initialize AI
    entity->ai.state = 0;
    entity->ai.timer = 0.0f;
    entity->ai.target_position = vec3_create(0, 0, 0);
    entity->ai.target_entity_id = -1;
    entity->ai.behavior_type = 0; // Idle
    
    return entity;
}

// === WEB-OPTIMIZED SYSTEMS ===

// Physics system optimized for web performance
void update_web_physics_system(float delta_time) {
    if (!g_game_state->physics_enabled) return;
    
    // Single-step integration for web performance
    const float sub_delta = delta_time / PHYSICS_SUBSTEPS;
    
    for (int step = 0; step < PHYSICS_SUBSTEPS; step++) {
        for (int i = 0; i < g_game_state->entity_count; i++) {
            WebEntity* entity = &g_game_state->entities[i];
            if (!entity->logic.active || entity->physics.is_kinematic || !entity->physics.enabled) {
                continue;
            }
            
            Transform* transform = &entity->transform;
            Physics* physics = &entity->physics;
            
            // Apply gravity
            if (physics->has_gravity) {
                transform->acceleration = vec3_add(transform->acceleration, g_game_state->gravity);
            }
            
            // Apply drag (simplified for web)
            float drag_factor = 1.0f - (physics->drag * sub_delta);
            transform->velocity = vec3_multiply_scalar(transform->velocity, drag_factor);
            
            // Verlet integration
            transform->velocity = vec3_add(transform->velocity, 
                vec3_multiply_scalar(transform->acceleration, sub_delta));
            transform->position = vec3_add(transform->position, 
                vec3_multiply_scalar(transform->velocity, sub_delta));
            
            // Reset acceleration
            transform->acceleration = vec3_create(0, 0, 0);
        }
    }
}

// Web-optimized particle system
void update_web_particle_system(float delta_time) {
    int active_particles = 0;
    
    for (int i = 0; i < g_game_state->particle_count; i++) {
        WebParticle* p = &g_game_state->particles[i];
        if (!p->active) continue;
        
        // Update physics
        p->velocity = vec3_add(p->velocity, vec3_multiply_scalar(g_game_state->gravity, delta_time));
        p->position = vec3_add(p->position, vec3_multiply_scalar(p->velocity, delta_time));
        
        // Update life
        p->life -= delta_time;
        if (p->life <= 0.0f) {
            p->active = false;
            continue;
        }
        
        // Update visual properties
        float life_ratio = p->life / 2.0f; // Assume 2s max life
        p->color[3] = life_ratio; // Alpha fade
        p->size *= 0.99f; // Size decay
        
        active_particles++;
    }
    
    // Compact particle array if needed (every 60 frames)
    static int compact_counter = 0;
    if (++compact_counter >= 60) {
        compact_counter = 0;
        
        int write_index = 0;
        for (int read_index = 0; read_index < g_game_state->particle_count; read_index++) {
            if (g_game_state->particles[read_index].active) {
                if (write_index != read_index) {
                    g_game_state->particles[write_index] = g_game_state->particles[read_index];
                }
                write_index++;
            }
        }
        g_game_state->particle_count = write_index;
    }
}

// Efficient collision detection for web
void update_web_collision_system() {
    // Simple spatial grid for better performance
    const int GRID_SIZE = 8;
    const float CELL_SIZE = CANVAS_WIDTH / GRID_SIZE;
    static int grid[GRID_SIZE][GRID_SIZE][16]; // Max 16 entities per cell
    static int grid_counts[GRID_SIZE][GRID_SIZE];
    
    // Clear grid
    memset(grid_counts, 0, sizeof(grid_counts));
    
    // Populate grid
    for (int i = 0; i < g_game_state->entity_count; i++) {
        WebEntity* entity = &g_game_state->entities[i];
        if (!entity->logic.active) continue;
        
        int grid_x = (int)(entity->transform.position.x / CELL_SIZE);
        int grid_y = (int)(entity->transform.position.y / CELL_SIZE);
        
        // Clamp to grid bounds
        grid_x = (grid_x < 0) ? 0 : (grid_x >= GRID_SIZE) ? GRID_SIZE-1 : grid_x;
        grid_y = (grid_y < 0) ? 0 : (grid_y >= GRID_SIZE) ? GRID_SIZE-1 : grid_y;
        
        if (grid_counts[grid_x][grid_y] < 16) {
            grid[grid_x][grid_y][grid_counts[grid_x][grid_y]++] = i;
        }
    }
    
    // Check collisions within grid cells
    for (int gx = 0; gx < GRID_SIZE; gx++) {
        for (int gy = 0; gy < GRID_SIZE; gy++) {
            int count = grid_counts[gx][gy];
            
            for (int i = 0; i < count; i++) {
                for (int j = i + 1; j < count; j++) {
                    int idx_a = grid[gx][gy][i];
                    int idx_b = grid[gx][gy][j];
                    
                    WebEntity* a = &g_game_state->entities[idx_a];
                    WebEntity* b = &g_game_state->entities[idx_b];
                    
                    Vector3 diff = vec3_add(a->transform.position, 
                        vec3_multiply_scalar(b->transform.position, -1.0f));
                    float dist_sq = vec3_magnitude_squared(diff);
                    
                    if (dist_sq < (32.0f * 32.0f)) { // Collision threshold
                        // Simple collision response
                        float bounce = 150.0f * (a->physics.bounciness + b->physics.bounciness) * 0.5f;
                        Vector3 normal = vec3_normalize_fast(diff);
                        Vector3 impulse = vec3_multiply_scalar(normal, bounce);
                        
                        a->transform.velocity = vec3_add(a->transform.velocity, impulse);
                        b->transform.velocity = vec3_add(b->transform.velocity, 
                            vec3_multiply_scalar(impulse, -1.0f));
                        
                        // Update score for player collisions
                        if (strcmp(a->logic.tag, "Player") == 0 || 
                            strcmp(b->logic.tag, "Player") == 0) {
                            g_game_state->score += 5;
                        }
                    }
                }
            }
        }
    }
}

// === WEB INPUT HANDLING ===

void process_web_input(float delta_time) {
    WebEntity* player = NULL;
    
    // Find player entity
    for (int i = 0; i < g_game_state->entity_count; i++) {
        if (strcmp(g_game_state->entities[i].logic.tag, "Player") == 0) {
            player = &g_game_state->entities[i];
            break;
        }
    }
    
    if (!player) return;
    
    WebInputState* input = &g_game_state->input;
    float move_speed = 400.0f; // Increased for web responsiveness
    
    Vector3 movement = vec3_create(0, 0, 0);
    
    // WASD movement
    if (input->keys[87]) movement.y -= 1.0f; // W
    if (input->keys[83]) movement.y += 1.0f; // S  
    if (input->keys[65]) movement.x -= 1.0f; // A
    if (input->keys[68]) movement.x += 1.0f; // D
    
    // Normalize diagonal movement
    float move_mag = vec3_magnitude(movement);
    if (move_mag > 0.1f) {
        movement = vec3_multiply_scalar(movement, move_speed / move_mag);
        player->transform.acceleration = vec3_add(player->transform.acceleration, movement);
    }
    
    // Touch input for mobile
    if (input->touch_active && input->touch_count > 0) {
        Vector3 touch_dir = vec3_create(
            input->touch_x - player->transform.position.x,
            input->touch_y - player->transform.position.y,
            0
        );
        
        float touch_dist = vec3_magnitude(touch_dir);
        if (touch_dist > 32.0f) { // Dead zone
            touch_dir = vec3_normalize_fast(touch_dir);
            Vector3 touch_force = vec3_multiply_scalar(touch_dir, move_speed * 0.5f);
            player->transform.acceleration = vec3_add(player->transform.acceleration, touch_force);
        }
    }
}

// === MAIN INITIALIZATION ===

void init_web_game_engine() {
    if (!allocate_web_game_state()) {
        printf("Failed to allocate web game state\n");
        return;
    }
    
    // Detect browser capabilities
    detect_web_capabilities();
    
    // Initialize camera
    g_game_state->camera_position = vec3_create(CANVAS_WIDTH/2, CANVAS_HEIGHT/2, -500);
    g_game_state->camera_target = vec3_create(CANVAS_WIDTH/2, CANVAS_HEIGHT/2, 0);
    g_game_state->camera_fov = 75.0f;
    
    // Initialize physics
    g_game_state->gravity = vec3_create(0, -490, 0); // Reduced gravity for web
    g_game_state->physics_enabled = true;
    
    // Initialize game state
    g_game_state->score = 0;
    g_game_state->level = 1;
    g_game_state->time_scale = 1.0f;
    g_game_state->paused = false;
    g_game_state->adaptive_quality = true;
    g_game_state->vsync_enabled = true;
    
    // Initialize performance tracking
    g_game_state->performance.last_frame_time = emscripten_get_now();
    g_game_state->performance.quality_level = 2; // Start with high quality
    
    // Create player
    WebEntity* player = create_web_entity(
        vec3_create(CANVAS_WIDTH/2, CANVAS_HEIGHT/2, 0), "Player");
    if (player) {
        player->physics.has_gravity = false; // Top-down view
        strcpy(player->logic.tag, "Player");
        player->renderer.color[0] = 0.3f;
        player->renderer.color[1] = 0.8f;
        player->renderer.color[2] = 1.0f;
    }
    
    // Create environment entities (reduced count for web)
    for (int i = 0; i < 30; i++) {
        Vector3 pos = vec3_create(
            (float)(rand() % (int)CANVAS_WIDTH),
            (float)(rand() % (int)CANVAS_HEIGHT),
            0
        );
        
        WebEntity* env = create_web_entity(pos, "Environment");
        if (env) {
            snprintf(env->logic.name, ENTITY_NAME_SIZE, "Obj_%d", i);
            strcpy(env->logic.tag, "Environment");
            env->physics.mass = 0.5f + ((float)rand() / RAND_MAX) * 1.5f;
            env->renderer.color[0] = 0.5f + ((float)rand() / RAND_MAX) * 0.5f;
            env->renderer.color[1] = 0.5f + ((float)rand() / RAND_MAX) * 0.5f;
            env->renderer.color[2] = 0.5f + ((float)rand() / RAND_MAX) * 0.5f;
        }
    }
    
    g_initialized = true;
    printf("Web Game Engine v1.12 initialized successfully\n");
    printf("WebGL2: %s, Hardware Accel: %s\n",
           g_game_state->graphics_caps.webgl2_available ? "Yes" : "No",
           g_game_state->graphics_caps.hardware_acceleration ? "Yes" : "No");
    printf("Entities: %d\n", g_game_state->entity_count);
}

// === MAIN UPDATE LOOP ===

void update_web_game_logic(double current_time) {
    if (g_game_state->paused) return;
    
    double frame_start = emscripten_get_now();
    BrowserPerformanceMetrics* perf = &g_game_state->performance;
    
    float delta_time = (float)(current_time - perf->last_frame_time) / 1000.0f * g_game_state->time_scale;
    perf->last_frame_time = current_time;
    
    // Cap delta time
    if (delta_time > 0.033f) delta_time = 0.033f;
    
    // Update FPS tracking
    perf->fps_counter++;
    perf->frame_accumulator += delta_time;
    if (perf->frame_accumulator >= 1.0) {
        perf->current_fps = perf->fps_counter / perf->frame_accumulator;
        perf->fps_counter = 0;
        perf->frame_accumulator = 0.0;
        
        if (g_game_state->debug_mode) {
            printf("FPS: %.1f, Frame: %.2fms, Quality: %d\n", 
                   perf->current_fps, perf->average_frame_time_ms, perf->quality_level);
        }
    }
    
    // Update systems based on quality level
    if (perf->quality_level >= 1) {
        update_web_physics_system(delta_time);
        update_web_collision_system();
    }
    
    if (perf->quality_level >= 2) {
        update_web_particle_system(delta_time);
    }
    
    // Always update input and basic entity logic
    process_web_input(delta_time);
    
    // Update camera to follow player smoothly
    WebEntity* player = &g_game_state->entities[0];
    if (player && player->logic.active) {
        float lerp_factor = 3.0f * delta_time;
        g_game_state->camera_target.x += (player->transform.position.x - g_game_state->camera_target.x) * lerp_factor;
        g_game_state->camera_target.y += (player->transform.position.y - g_game_state->camera_target.y) * lerp_factor;
    }
    
    // Calculate frame time and adjust quality
    double frame_end = emscripten_get_now();
    float frame_time_ms = (float)(frame_end - frame_start);
    perf->average_frame_time_ms = perf->average_frame_time_ms * 0.9f + frame_time_ms * 0.1f;
    
    update_adaptive_quality(frame_time_ms);
}

// === WASM EXPORTS FOR WEB ===

EMSCRIPTEN_KEEPALIVE
int wasm_init_web_game() {
    printf("Initializing Web Game Engine v1.12\n");
    init_web_game_engine();
    return g_initialized ? 1 : 0;
}

EMSCRIPTEN_KEEPALIVE
void wasm_update_web_frame(double current_time) {
    if (!g_initialized || !g_game_state) return;
    update_web_game_logic(current_time);
}

EMSCRIPTEN_KEEPALIVE
void wasm_handle_web_key(int key_code, int pressed) {
    if (!g_initialized || !g_game_state) return;
    
    if (key_code >= 0 && key_code < 256) {
        g_game_state->input.keys[key_code] = (pressed != 0);
    }
    
    // Special keys
    if (pressed) {
        switch (key_code) {
            case 32: // Space
                g_game_state->paused = !g_game_state->paused;
                break;
            case 192: // Tilde (~) for debug
                g_game_state->debug_mode = !g_game_state->debug_mode;
                break;
        }
    }
}

EMSCRIPTEN_KEEPALIVE
void wasm_handle_mouse(float x, float y, float delta_x, float delta_y) {
    if (!g_game_state) return;
    
    g_game_state->input.mouse_x = x;
    g_game_state->input.mouse_y = y;
    g_game_state->input.mouse_delta_x = delta_x;
    g_game_state->input.mouse_delta_y = delta_y;
}

EMSCRIPTEN_KEEPALIVE
void wasm_handle_touch(float x, float y, int active, int count) {
    if (!g_game_state) return;
    
    g_game_state->input.touch_x = x;
    g_game_state->input.touch_y = y;
    g_game_state->input.touch_active = (active != 0);
    g_game_state->input.touch_count = count;
}

EMSCRIPTEN_KEEPALIVE
int wasm_get_score() {
    return g_game_state ? g_game_state->score : 0;
}

EMSCRIPTEN_KEEPALIVE
int wasm_get_entity_count() {
    return g_game_state ? g_game_state->entity_count : 0;
}

EMSCRIPTEN_KEEPALIVE
int wasm_get_particle_count() {
    return g_game_state ? g_game_state->particle_count : 0;
}

EMSCRIPTEN_KEEPALIVE
float wasm_get_fps() {
    return g_game_state ? g_game_state->performance.current_fps : 0.0f;
}

EMSCRIPTEN_KEEPALIVE
float wasm_get_frame_time() {
    return g_game_state ? g_game_state->performance.average_frame_time_ms : 0.0f;
}

EMSCRIPTEN_KEEPALIVE
void wasm_set_quality(int quality) {
    if (!g_game_state) return;
    
    g_game_state->performance.quality_level = (quality < 0) ? 0 : (quality > 2
