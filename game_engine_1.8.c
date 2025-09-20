#include <emscripten.h>
#include <emscripten/html5.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <stdbool.h>

// Web-Optimized Game Engine for Deplauncher 1.8 - Classic Edition
// Lightweight engine optimized for web browsers with excellent performance

// === WEB-SPECIFIC CONSTANTS ===
#define MAX_ENTITIES 800        // Optimized for web browsers
#define MAX_PARTICLES 200       // Limited for consistent performance
#define CANVAS_WIDTH 800.0f
#define CANVAS_HEIGHT 600.0f
#define ENTITY_NAME_SIZE 24     // Reduced memory footprint
#define COLLISION_RADIUS 28.0f  // Slightly smaller for better responsiveness
#define MOVE_SPEED 180.0f       // Adjusted for web feel
#define FRICTION 0.92f          // More responsive friction
#define TARGET_FPS 60.0f
#define MAX_FRAME_TIME_MS 16.67f

// === WEB-OPTIMIZED STRUCTURES ===

// Compact 2D Vector for web performance
typedef struct __attribute__((packed)) {
    float x, y;
} WebVector2;

// Lightweight particle for web
typedef struct __attribute__((packed)) {
    WebVector2 position;
    WebVector2 velocity;
    float life;
    float size;
    unsigned char color[3]; // RGB only, alpha calculated from life
    bool active;
} WebParticle;

// Web-optimized entity structure
typedef struct __attribute__((aligned(32))) {
    WebVector2 position;
    WebVector2 velocity;
    float rotation;
    short health;           // 16-bit for memory efficiency
    short max_health;
    int texture_id;
    bool active;
    char name[ENTITY_NAME_SIZE];
    char tag[12];          // Shorter tag for memory efficiency
} WebEntity;

// Web-specific performance tracking
typedef struct {
    double last_frame_time;
    double frame_accumulator;
    int fps_counter;
    float current_fps;
    float average_frame_time_ms;
    int quality_level;      // 0=low, 1=medium, 2=high
    bool adaptive_quality;
    int quality_cooldown;
} WebPerformanceMetrics;

// Browser-specific input state
typedef struct {
    bool keys[256];
    WebVector2 mouse_pos;
    WebVector2 mouse_delta;
    bool mouse_buttons[3];
    WebVector2 touch_pos;
    bool touch_active;
    int touch_count;
} WebInputState;

// Camera system for web
typedef struct {
    WebVector2 position;
    WebVector2 target;
    float zoom;
    float follow_speed;
    float shake_intensity;
    float shake_duration;
} WebCamera;

// Main web game state
typedef struct {
    // Core systems
    WebEntity entities[MAX_ENTITIES];
    WebParticle particles[MAX_PARTICLES];
    int entity_count;
    int particle_count;
    
    // Web-specific systems
    WebCamera camera;
    WebPerformanceMetrics performance;
    WebInputState input;
    
    // Game state
    int score;
    int level;
    bool paused;
    bool debug_mode;
    
    // Web optimization
    bool vsync_enabled;
    bool low_power_mode;
    float time_scale;
} WebGameState;

// === GLOBAL STATE ===
static WebGameState* g_web_game_state = NULL;
static bool g_web_initialized = false;

// Performance monitoring
static int g_quality_adjustment_timer = 0;
static float g_frame_time_buffer[60]; // Rolling average buffer
static int g_frame_time_index = 0;

// === UTILITY FUNCTIONS ===

// Fast Vector2 operations optimized for web
WebVector2 web_vec2_create(float x, float y) {
    WebVector2 v = {x, y};
    return v;
}

WebVector2 web_vec2_add(WebVector2 a, WebVector2 b) {
    return (WebVector2){a.x + b.x, a.y + b.y};
}

WebVector2 web_vec2_subtract(WebVector2 a, WebVector2 b) {
    return (WebVector2){a.x - b.x, a.y - b.y};
}

WebVector2 web_vec2_multiply_scalar(WebVector2 v, float scalar) {
    return (WebVector2){v.x * scalar, v.y * scalar};
}

float web_vec2_magnitude_squared(WebVector2 v) {
    return v.x * v.x + v.y * v.y;
}

float web_vec2_magnitude(WebVector2 v) {
    return sqrtf(web_vec2_magnitude_squared(v));
}

WebVector2 web_vec2_normalize_fast(WebVector2 v) {
    float inv_mag = 1.0f / sqrtf(web_vec2_magnitude_squared(v) + 1e-6f);
    return web_vec2_multiply_scalar(v, inv_mag);
}

float web_vec2_distance_squared(WebVector2 a, WebVector2 b) {
    WebVector2 diff = web_vec2_subtract(a, b);
    return web_vec2_magnitude_squared(diff);
}

// Fast math utilities
float web_lerp(float a, float b, float t) {
    return a + t * (b - a);
}

float web_clamp(float value, float min_val, float max_val) {
    return (value < min_val) ? min_val : (value > max_val) ? max_val : value;
}

// === WEB PERFORMANCE SYSTEM ===

void update_web_performance_metrics(double current_time) {
    WebPerformanceMetrics* perf = &g_web_game_state->performance;
    
    // Calculate frame time
    float frame_time_ms = (float)((current_time - perf->last_frame_time));
    perf->last_frame_time = current_time;
    
    // Update rolling average
    g_frame_time_buffer[g_frame_time_index] = frame_time_ms;
    g_frame_time_index = (g_frame_time_index + 1) % 60;
    
    // Calculate average frame time
    float total_time = 0.0f;
    for (int i = 0; i < 60; i++) {
        total_time += g_frame_time_buffer[i];
    }
    perf->average_frame_time_ms = total_time / 60.0f;
    
    // Update FPS
    perf->fps_counter++;
    perf->frame_accumulator += frame_time_ms / 1000.0;
    if (perf->frame_accumulator >= 1.0) {
        perf->current_fps = perf->fps_counter / perf->frame_accumulator;
        perf->fps_counter = 0;
        perf->frame_accumulator = 0.0;
        
        if (g_web_game_state->debug_mode) {
            printf("Web FPS: %.1f, Frame Time: %.2fms, Quality: %d\n",
                   perf->current_fps, perf->average_frame_time_ms, perf->quality_level);
        }
    }
    
    // Adaptive quality adjustment
    if (perf->adaptive_quality && g_quality_adjustment_timer <= 0) {
        if (perf->average_frame_time_ms > MAX_FRAME_TIME_MS * 1.2f) {
    // Adaptive quality adjustment
    if (perf->adaptive_quality && g_quality_adjustment_timer <= 0) {
        if (perf->average_frame_time_ms > MAX_FRAME_TIME_MS * 1.2f) {
            if (perf->quality_level > 0) {
                perf->quality_level--;
                g_quality_adjustment_timer = 180; // 3 second cooldown
                printf("Quality reduced to %d (frame time: %.2fms)\n", 
                       perf->quality_level, perf->average_frame_time_ms);
            }
        } else if (perf->average_frame_time_ms < MAX_FRAME_TIME_MS * 0.8f) {
            if (perf->quality_level < 2) {
                perf->quality_level++;
                g_quality_adjustment_timer = 300; // 5 second cooldown for increases
                printf("Quality increased to %d (frame time: %.2fms)\n", 
                       perf->quality_level, perf->average_frame_time_ms);
            }
        }
    }
    
    if (g_quality_adjustment_timer > 0) {
        g_quality_adjustment_timer--;
    }
}

// === MEMORY MANAGEMENT ===

bool allocate_web_game_state() {
    g_web_game_state = (WebGameState*)aligned_alloc(32, sizeof(WebGameState));
    if (g_web_game_state) {
        memset(g_web_game_state, 0, sizeof(WebGameState));
        return true;
    }
    return false;
}

void deallocate_web_game_state() {
    if (g_web_game_state) {
        free(g_web_game_state);
        g_web_game_state = NULL;
    }
}

// === ENTITY MANAGEMENT ===

WebEntity* create_web_entity(WebVector2 position, int texture_id, const char* name) {
    if (!g_web_game_state || g_web_game_state->entity_count >= MAX_ENTITIES) {
        return NULL;
    }
    
    WebEntity* entity = &g_web_game_state->entities[g_web_game_state->entity_count++];
    
    entity->position = position;
    entity->velocity = web_vec2_create(0, 0);
    entity->rotation = 0.0f;
    entity->texture_id = texture_id;
    entity->active = true;
    entity->health = 100;
    entity->max_health = 100;
    strncpy(entity->name, name, ENTITY_NAME_SIZE - 1);
    entity->name[ENTITY_NAME_SIZE - 1] = '\0';
    strcpy(entity->tag, "Default");
    
    return entity;
}

WebEntity* find_web_entity_by_tag(const char* tag) {
    if (!g_web_game_state) return NULL;
    
    for (int i = 0; i < g_web_game_state->entity_count; i++) {
        WebEntity* entity = &g_web_game_state->entities[i];
        if (entity->active && strcmp(entity->tag, tag) == 0) {
            return entity;
        }
    }
    return NULL;
}

// === WEB-OPTIMIZED SYSTEMS ===

void update_web_entity_physics(WebEntity* entity, float delta_time) {
    if (!entity || !entity->active) return;
    
    // Apply velocity
    entity->position = web_vec2_add(entity->position, 
        web_vec2_multiply_scalar(entity->velocity, delta_time));
    
    // Apply rotation (visual effect)
    entity->rotation += delta_time * 60.0f; // 60 degrees per second
    if (entity->rotation > 360.0f) entity->rotation -= 360.0f;
    
    // Web-friendly boundary wrapping
    if (entity->position.x < -32.0f) entity->position.x = CANVAS_WIDTH + 32.0f;
    if (entity->position.x > CANVAS_WIDTH + 32.0f) entity->position.x = -32.0f;
    if (entity->position.y < -32.0f) entity->position.y = CANVAS_HEIGHT + 32.0f;
    if (entity->position.y > CANVAS_HEIGHT + 32.0f) entity->position.y = -32.0f;
    
    // Apply friction
    entity->velocity = web_vec2_multiply_scalar(entity->velocity, FRICTION);
}

void update_web_entity_ai(WebEntity* entity, double current_time) {
    if (!entity || !entity->active || strcmp(entity->tag, "Player") == 0) return;
    
    // Simple but effective AI for web
    float time_factor = (float)(current_time * 0.0008); // Slower movement
    float ai_speed = 40.0f;
    
    // Create interesting movement patterns
    float pattern = sinf(time_factor + entity->texture_id * 2.0f);
    WebVector2 ai_velocity = web_vec2_create(
        sinf(time_factor) * pattern * ai_speed,
        cosf(time_factor * 1.3f) * pattern * ai_speed
    );
    
    entity->velocity = web_vec2_add(entity->velocity, 
        web_vec2_multiply_scalar(ai_velocity, 0.1f)); // Smooth blending
}

// === WEB PARTICLE SYSTEM ===

void create_web_particle_explosion(WebVector2 position, int count) {
    count = (count > 20) ? 20 : count; // Limit for web performance
    
    for (int i = 0; i < count && g_web_game_state->particle_count < MAX_PARTICLES; i++) {
        WebParticle* p = &g_web_game_state->particles[g_web_game_state->particle_count++];
        
        // Random explosion direction
        float angle = ((float)rand() / RAND_MAX) * 2.0f * M_PI;
        float speed = 60.0f + ((float)rand() / RAND_MAX) * 80.0f;
        
        p->position = position;
        p->velocity = web_vec2_create(cosf(angle) * speed, sinf(angle) * speed);
        p->life = 1.0f + ((float)rand() / RAND_MAX) * 1.5f;
        p->size = 2.0f + ((float)rand() / RAND_MAX) * 3.0f;
        p->active = true;
        
        // Orange explosion colors
        p->color[0] = 255; // Red
        p->color[1] = (unsigned char)(180 + (rand() % 75)); // Green
        p->color[2] = (unsigned char)(50 + (rand() % 100));  // Blue
    }
}

void update_web_particle_system(float delta_time) {
    int active_count = 0;
    
    for (int i = 0; i < g_web_game_state->particle_count; i++) {
        WebParticle* p = &g_web_game_state->particles[i];
        if (!p->active) continue;
        
        // Apply simple gravity
        p->velocity.y += 120.0f * delta_time;
        
        // Update position
        p->position = web_vec2_add(p->position, web_vec2_multiply_scalar(p->velocity, delta_time));
        
        // Update life
        p->life -= delta_time;
        if (p->life <= 0.0f) {
            p->active = false;
            continue;
        }
        
        // Visual updates
        p->size *= 0.98f; // Gradual size reduction
        active_count++;
    }
    
    // Compact particle array periodically (every 120 frames = 2 seconds)
    static int compact_timer = 0;
    if (++compact_timer >= 120) {
        compact_timer = 0;
        
        int write_index = 0;
        for (int read_index = 0; read_index < g_web_game_state->particle_count; read_index++) {
            if (g_web_game_state->particles[read_index].active) {
                if (write_index != read_index) {
                    g_web_game_state->particles[write_index] = g_web_game_state->particles[read_index];
                }
                write_index++;
            }
        }
        g_web_game_state->particle_count = write_index;
    }
}

// === WEB COLLISION SYSTEM ===

void update_web_collision_system() {
    // Optimized collision detection for web
    const float collision_radius_sq = COLLISION_RADIUS * COLLISION_RADIUS;
    
    for (int i = 0; i < g_web_game_state->entity_count; i++) {
        WebEntity* entity_a = &g_web_game_state->entities[i];
        if (!entity_a->active) continue;
        
        for (int j = i + 1; j < g_web_game_state->entity_count; j++) {
            WebEntity* entity_b = &g_web_game_state->entities[j];
            if (!entity_b->active) continue;
            
            float dist_sq = web_vec2_distance_squared(entity_a->position, entity_b->position);
            
            if (dist_sq < collision_radius_sq) {
                // Collision response
                WebVector2 direction = web_vec2_subtract(entity_a->position, entity_b->position);
                float distance = sqrtf(dist_sq);
                
                if (distance > 0.1f) {
                    direction = web_vec2_multiply_scalar(direction, 1.0f / distance);
                    
                    // Separate entities
                    float overlap = COLLISION_RADIUS - distance;
                    WebVector2 separation = web_vec2_multiply_scalar(direction, overlap * 0.5f);
                    
                    entity_a->position = web_vec2_add(entity_a->position, separation);
                    entity_b->position = web_vec2_subtract(entity_b->position, separation);
                    
                    // Apply bounce effect
                    float bounce_force = 120.0f;
                    WebVector2 bounce_impulse = web_vec2_multiply_scalar(direction, bounce_force);
                    
                    entity_a->velocity = web_vec2_add(entity_a->velocity, bounce_impulse);
                    entity_b->velocity = web_vec2_subtract(entity_b->velocity, bounce_impulse);
                    
                    // Create collision effect
                    if (g_web_game_state->performance.quality_level >= 2) {
                        WebVector2 collision_point = web_vec2_multiply_scalar(
                            web_vec2_add(entity_a->position, entity_b->position), 0.5f);
                        create_web_particle_explosion(collision_point, 3);
                    }
                    
                    // Score for player collisions
                    if (strcmp(entity_a->tag, "Player") == 0 || 
                        strcmp(entity_b->tag, "Player") == 0) {
                        g_web_game_state->score += 10;
                    }
                }
            }
        }
    }
}

// === WEB CAMERA SYSTEM ===

void init_web_camera() {
    WebCamera* cam = &g_web_game_state->camera;
    cam->position = web_vec2_create(CANVAS_WIDTH / 2, CANVAS_HEIGHT / 2);
    cam->target = web_vec2_create(CANVAS_WIDTH / 2, CANVAS_HEIGHT / 2);
    cam->zoom = 1.0f;
    cam->follow_speed = 4.0f; // Responsive camera
    cam->shake_intensity = 0.0f;
    cam->shake_duration = 0.0f;
}

void update_web_camera(float delta_time) {
    WebCamera* cam = &g_web_game_state->camera;
    
    // Follow player
    WebEntity* player = find_web_entity_by_tag("Player");
    if (player) {
        cam->target = player->position;
    }
    
    // Smooth camera movement
    float lerp_factor = 1.0f - expf(-cam->follow_speed * delta_time);
    cam->position.x = web_lerp(cam->position.x, cam->target.x, lerp_factor);
    cam->position.y = web_lerp(cam->position.y, cam->target.y, lerp_factor);
    
    // Camera shake effect
    if (cam->shake_duration > 0.0f) {
        cam->shake_duration -= delta_time;
        float shake_x = ((float)rand() / RAND_MAX - 0.5f) * cam->shake_intensity;
        float shake_y = ((float)rand() / RAND_MAX - 0.5f) * cam->shake_intensity;
        cam->position.x += shake_x;
        cam->position.y += shake_y;
        
        cam->shake_intensity *= 0.95f; // Decay
    }
}

void trigger_camera_shake(float intensity, float duration) {
    WebCamera* cam = &g_web_game_state->camera;
    cam->shake_intensity = intensity;
    cam->shake_duration = duration;
}

// === WEB INPUT SYSTEM ===

void process_web_input(float delta_time) {
    WebEntity* player = find_web_entity_by_tag("Player");
    if (!player) return;
    
    WebInputState* input = &g_web_game_state->input;
    WebVector2 movement = web_vec2_create(0, 0);
    
    // Keyboard input (WASD + Arrow keys)
    if (input->keys[87] || input->keys[38]) movement.y -= 1.0f; // W or Up
    if (input->keys[83] || input->keys[40]) movement.y += 1.0f; // S or Down
    if (input->keys[65] || input->keys[37]) movement.x -= 1.0f; // A or Left
    if (input->keys[68] || input->keys[39]) movement.x += 1.0f; // D or Right
    
    // Normalize diagonal movement
    float move_mag = web_vec2_magnitude(movement);
    if (move_mag > 0.1f) {
        movement = web_vec2_multiply_scalar(movement, MOVE_SPEED / move_mag);
        player->velocity = web_vec2_add(player->velocity, 
            web_vec2_multiply_scalar(movement, delta_time));
    }
    
    // Touch input for mobile
    if (input->touch_active && input->touch_count > 0) {
        WebVector2 touch_dir = web_vec2_subtract(input->touch_pos, player->position);
        float touch_dist = web_vec2_magnitude(touch_dir);
        
        if (touch_dist > 32.0f) { // Dead zone
            touch_dir = web_vec2_multiply_scalar(touch_dir, 1.0f / touch_dist);
            WebVector2 touch_force = web_vec2_multiply_scalar(touch_dir, MOVE_SPEED * 0.6f);
            player->velocity = web_vec2_add(player->velocity, 
                web_vec2_multiply_scalar(touch_force, delta_time));
        }
    }
}

// === CLEANUP SYSTEM ===

void cleanup_web_entities() {
    int write_index = 0;
    for (int read_index = 0; read_index < g_web_game_state->entity_count; read_index++) {
        WebEntity* entity = &g_web_game_state->entities[read_index];
        if (entity->active && entity->health > 0) {
            if (write_index != read_index) {
                g_web_game_state->entities[write_index] = *entity;
            }
            write_index++;
        }
    }
    g_web_game_state->entity_count = write_index;
}

// === MAIN GAME FUNCTIONS ===

void init_web_game_engine() {
    if (!allocate_web_game_state()) {
        printf("Failed to allocate web game state\n");
        return;
    }
    
    // Initialize performance tracking
    WebPerformanceMetrics* perf = &g_web_game_state->performance;
    perf->last_frame_time = emscripten_get_now();
    perf->quality_level = 2; // Start with high quality
    perf->adaptive_quality = true;
    
    // Initialize camera
    init_web_camera();
    
    // Initialize game state
    g_web_game_state->score = 0;
    g_web_game_state->level = 1;
    g_web_game_state->paused = false;
    g_web_game_state->debug_mode = false;
    g_web_game_state->vsync_enabled = true;
    g_web_game_state->time_scale = 1.0f;
    
    // Create player
    WebEntity* player = create_web_entity(
        web_vec2_create(CANVAS_WIDTH / 2, CANVAS_HEIGHT / 2), 0, "Player");
    if (player) {
        strcpy(player->tag, "Player");
    }
    
    // Create environment entities (reduced for web)
    for (int i = 0; i < 15; i++) {
        WebVector2 pos = web_vec2_create(
            (float)(rand() % (int)(CANVAS_WIDTH - 100)) + 50,
            (float)(rand() % (int)(CANVAS_HEIGHT - 100)) + 50
        );
        
        WebEntity* env = create_web_entity(pos, 1 + (rand() % 3), "Environment");
        if (env) {
            snprintf(env->name, ENTITY_NAME_SIZE, "Obj_%d", i);
            strcpy(env->tag, "Environment");
            
            // Give initial velocity for dynamic gameplay
            env->velocity = web_vec2_create(
                ((float)rand() / RAND_MAX - 0.5f) * 40.0f,
                ((float)rand() / RAND_MAX - 0.5f) * 40.0f
            );
        }
    }
    
    g_web_initialized = true;
    printf("Web Game Engine v1.8 Classic initialized successfully\n");
    printf("Entities: %d, Adaptive Quality: %s\n", 
           g_web_game_state->entity_count, 
           perf->adaptive_quality ? "Enabled" : "Disabled");
}

void update_web_game_logic(double current_time) {
    if (!g_web_game_state || g_web_game_state->paused) return;
    
    // Update performance metrics first
    update_web_performance_metrics(current_time);
    
    float delta_time = (float)(g_web_game_state->performance.average_frame_time_ms / 1000.0) * g_web_game_state->time_scale;
    delta_time = web_clamp(delta_time, 0.0f, 0.033f); // Cap at 30 FPS minimum
    
    // Process input
    process_web_input(delta_time);
    
    // Update entities based on quality level
    for (int i = 0; i < g_web_game_state->entity_count; i++) {
        WebEntity* entity = &g_web_game_state->entities[i];
        if (entity->active) {
            update_web_entity_physics(entity, delta_time);
            
            // AI only on medium/high quality
            if (g_web_game_state->performance.quality_level >= 1) {
                update_web_entity_ai(entity, current_time);
            }
        }
    }
    
    // Systems based on quality level
    if (g_web_game_state->performance.quality_level >= 1) {
        update_web_collision_system();
    }
    
    if (g_web_game_state->performance.quality_level >= 2) {
        update_web_particle_system(delta_time);
    }
    
    // Always update camera
    update_web_camera(delta_time);
    
    // Cleanup periodically
    static int cleanup_timer = 0;
    if (++cleanup_timer >= 300) { // Every 5 seconds at 60 FPS
        cleanup_timer = 0;
        cleanup_web_entities();
    }
}

// === WASM EXPORTS ===

EMSCRIPTEN_KEEPALIVE
int wasm_init_web_game() {
    printf("Initializing Web Game Engine v1.8 Classic Edition\n");
    init_web_game_engine();
    return g_web_initialized ? 1 : 0;
}

EMSCRIPTEN_KEEPALIVE
void wasm_update_web_frame(double current_time) {
    if (!g_web_initialized || !g_web_game_state) return;
    update_web_game_logic(current_time);
}

EMSCRIPTEN_KEEPALIVE
void wasm_handle_web_key(int key_code, int pressed) {
    if (!g_web_initialized || !g_web_game_state) return;
    
    if (key_code >= 0 && key_code < 256) {
        g_web_game_state->input.keys[key_code] = (pressed != 0);
    }
    
    // Special keys
    if (pressed) {
        switch (key_code) {
            case 32: // Space
                g_web_game_state->paused = !g_web_game_state->paused;
                break;
            case 192: // Tilde (~) for debug
                g_web_game_state->debug_mode = !g_web_game_state->debug_mode;
                break;
        }
    }
}

EMSCRIPTEN_KEEPALIVE
void wasm_handle_web_mouse(float x, float y, float delta_x, float delta_y) {
    if (!g_web_game_state) return;
    
    g_web_game_state->input.mouse_pos = web_vec2_create(x, y);
    g_web_game_state->input.mouse_delta = web_vec2_create(delta_x, delta_y);
}

EMSCRIPTEN_KEEPALIVE
void wasm_handle_web_touch(float x, float y, int active, int count) {
    if (!g_web_game_state) return;
    
    g_web_game_state->input.touch_pos = web_vec2_create(x, y);
    g_web_game_state->input.touch_active = (active != 0);
    g_web_game_state->input.touch_count = count;
}

EMSCRIPTEN_KEEPALIVE
int wasm_get_web_score() {
    return g_web_game_state ? g_web_game_state->score : 0;
}

EMSCRIPTEN_KEEPALIVE
int wasm_get_web_entity_count() {
    return g_web_game_state ? g_web_game_state->entity_count : 0;
}

EMSCRIPTEN_KEEPALIVE
int wasm_get_web_particle_count() {
    return g_web_game_state ? g_web_game_state->particle_count : 0;
}

EMSCRIPTEN_KEEPALIVE
float wasm_get_web_fps() {
    return g_web_game_state ? g_web_game_state->performance.current_fps : 0.0f;
}

EMSCRIPTEN_KEEPALIVE
float wasm_get_web_frame_time() {
    return g_web_game_state ? g_web_game_state->performance.average_frame_time_ms : 0.0f;
}

EMSCRIPTEN_KEEPALIVE
int wasm_get_web_quality_level() {
    return g_web_game_state ? g_web_game_state->performance.quality_level : 2;
}

EMSCRIPTEN_KEEPALIVE
void wasm_set_web_quality(int quality) {
    if (!g_web_game_state) return;
    
    g_web_game_state->performance.quality_level = web_clamp(quality, 0, 2);
    g_web_game_state->performance.adaptive_quality = false;
    printf("Quality manually set to %d\n", g_web_game_state->performance.quality_level);
}

EMSCRIPTEN_KEEPALIVE
void wasm_enable_web_adaptive_quality(int enabled) {
    if (!g_web_game_state) return;
    g_web_game_state->performance.adaptive_quality = (enabled != 0);
}

// Export render data for web canvas/WebGL
EMSCRIPTEN_KEEPALIVE
float* wasm_get_web_entity_data() {
    if (!g_web_game_state) return NULL;
    
    static float entity_buffer[MAX_ENTITIES * 8]; // pos + vel + rotation + color
    int buffer_index = 0;
    
    for (int i = 0; i < g_web_game_state->entity_count; i++) {
        WebEntity* entity = &g_web_game_state->entities[i];
        if (!entity->active) continue;
        
        entity_buffer[buffer_index++] = entity->position.x;
        entity_buffer[buffer_index++] = entity->position.y;
        entity_buffer[buffer_index++] = entity->velocity.x;
        entity_buffer[buffer_index++] = entity->velocity.y;
        entity_buffer[buffer_index++] = entity->rotation;
        entity_buffer[buffer_index++] = (float)entity->texture_id;
        entity_buffer[buffer_index++] = (float)entity->health / (float)entity->max_health; // Health ratio
        entity_buffer[buffer_index++] = (strcmp(entity->tag, "Player") == 0) ? 1.0f : 0.0f; // Is player flag
        
        if (buffer_index >= MAX_ENTITIES * 8 - 8) break;
    }
    
    return entity_buffer;
}

EMSCRIPTEN_KEEPALIVE
float* wasm_get_web_particle_data() {
    if (!g_web_game_state) return NULL;
    
    static float particle_buffer[MAX_PARTICLES * 6]; // pos + size + color
    int buffer_index = 0;
    
    for (int i = 0; i < g_web_game_state->particle_count; i++) {
        WebParticle* p = &g_web_game_state->particles[i];
        if (!p->active) continue;
        
        particle_buffer[buffer_index++] = p->position.x;
        particle_buffer[buffer_index++] = p->position.y;
        particle_buffer[buffer_index++] = p->size;
        particle_buffer[buffer_index++] = p->color[0] / 255.0f; // Normalize to 0-1
        particle_buffer[buffer_index++] = p->color[1] / 255.0f;
        particle_buffer[buffer_index++] = p->color[2] / 255.0f;
        
        if (buffer_index >= MAX_PARTICLES * 6 - 6) break;
    }
    
    return particle_buffer;
}

EMSCRIPTEN_KEEPALIVE
float* wasm_get_web_camera_data() {
    if (!g_web_game_state) return NULL;
    
    static float camera_buffer[5];
    WebCamera* cam = &g_web_game_state->camera;
    
    camera_buffer[0] = cam->position.x;
    camera_buffer[1] = cam->position.y;
    camera_buffer[2] = cam->zoom;
    camera_buffer[3] = CANVAS_WIDTH;
    camera_buffer[4] = CANVAS_HEIGHT;
    
    return camera_buffer;
}

EMSCRIPTEN_KEEPALIVE
void wasm_create_web_explosion(float x, float y, int count) {
    if (!g_web_game_state) return;
    
    create_web_particle_explosion(web_vec2_create(x, y), count);
    trigger_camera_shake(8.0f, 0.3f); // Add screen shake
}

EMSCRIPTEN_KEEPALIVE
void wasm_add_web_entity(float x, float y, int texture_id, const char* name, const char* tag) {
    if (!g_web_game_state) return;
    
    WebEntity* entity = create_web_entity(web_vec2_create(x, y), texture_id, name);
    if (entity && tag && strlen(tag) > 0) {
        strncpy(entity->tag, tag, sizeof(entity->tag) - 1);
        entity->tag[sizeof(entity->tag) - 1] = '\0';
    }
}

EMSCRIPTEN_KEEPALIVE
void wasm_reset_web_game() {
    if (!g_web_game_state) return;
    
    printf("Resetting web game state\n");
    
    // Clear entities and particles
    g_web_game_state->entity_count = 0;
    g_web_game_state->particle_count = 0;
    
    // Reset game state
    g_web_game_state->score = 0;
    g_web_game_state->level = 1;
    g_web_game_state->paused = false;
    
    // Reset performance metrics
    g_web_game_state->performance.fps_counter = 0;
    g_web_game_state->performance.frame_accumulator = 0.0;
    g_quality_adjustment_timer = 0;
    
    // Recreate initial entities
    WebEntity* player = create_web_entity(
        web_vec2_create(CANVAS_WIDTH / 2, CANVAS_HEIGHT / 2), 0, "Player");
    if (player) {
        strcpy(player->tag, "Player");
    }
    
    // Create fewer entities for reset
    for (int i = 0; i < 10; i++) {
        WebVector2 pos = web_vec2_create(
            (float)(rand() % (int)(CANVAS_WIDTH - 100)) + 50,
            (float)(rand() % (int)(CANVAS_HEIGHT - 100)) + 50
        );
        
        WebEntity* env = create_web_entity(pos, 1 + (rand() % 3), "Environment");
        if (env) {
            snprintf(env->name, ENTITY_NAME_SIZE, "Obj_%d", i);
            strcpy(env->tag, "Environment");
        }
    }
}

EMSCRIPTEN_KEEPALIVE
void wasm_cleanup_web() {
    printf("Web Game Engine v1.8 cleaned up\n");
    deallocate_web_game_state();
    g_web_initialized = false;
}
