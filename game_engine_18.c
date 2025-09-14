#include <emscripten.h>
#include <emscripten/html5.h>
#include <emscripten/threading.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <stdbool.h>

// Game Engine for Deplauncher 1.8 - Classic Edition
// Optimized for low-end devices with basic features

#define MAX_ENTITIES 1000
#define MAX_TEXTURES 100
#define CANVAS_WIDTH 800
#define CANVAS_HEIGHT 600

// Entity structure for classic gameplay
typedef struct {
    float x, y, z;
    float velocity_x, velocity_y;
    float rotation;
    int texture_id;
    bool active;
    int health;
    char name[32];
} Entity;

// Game state structure
typedef struct {
    Entity entities[MAX_ENTITIES];
    int entity_count;
    float camera_x, camera_y;
    int score;
    bool paused;
    double last_frame_time;
    int fps_counter;
    double fps_timer;
} GameState;

// Global game state
static GameState* g_game_state = NULL;
static bool g_initialized = false;

// Function declarations
void init_game_engine();
void update_game_logic(double current_time);
void render_frame();
void handle_input(int key_code, bool pressed);
Entity* create_entity(float x, float y, int texture_id);
void update_entity(Entity* entity, float delta_time);
void collision_detection();
void cleanup_inactive_entities();

// WebAssembly exports
EMSCRIPTEN_KEEPALIVE
int wasm_init_game() {
    printf("Initializing Game Engine v1.8 Classic Edition\n");
    init_game_engine();
    return g_initialized ? 1 : 0;
}

EMSCRIPTEN_KEEPALIVE
void wasm_update_frame(double current_time) {
    if (!g_initialized || !g_game_state) return;
    
    update_game_logic(current_time);
    render_frame();
}

EMSCRIPTEN_KEEPALIVE
void wasm_handle_key(int key_code, int pressed) {
    if (!g_initialized) return;
    handle_input(key_code, pressed == 1);
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
void wasm_pause_game(int paused) {
    if (g_game_state) {
        g_game_state->paused = paused == 1;
    }
}

// Game engine implementation
void init_game_engine() {
    // Allocate game state with WASM GC consideration
    g_game_state = (GameState*)calloc(1, sizeof(GameState));
    if (!g_game_state) {
        printf("Failed to allocate game state memory\n");
        return;
    }
    
    // Initialize default values
    g_game_state->camera_x = CANVAS_WIDTH / 2;
    g_game_state->camera_y = CANVAS_HEIGHT / 2;
    g_game_state->score = 0;
    g_game_state->paused = false;
    g_game_state->entity_count = 0;
    g_game_state->last_frame_time = emscripten_get_now();
    g_game_state->fps_counter = 0;
    g_game_state->fps_timer = 0;
    
    // Initialize entities array
    memset(g_game_state->entities, 0, sizeof(g_game_state->entities));
    
    // Create initial player entity
    Entity* player = create_entity(CANVAS_WIDTH / 2, CANVAS_HEIGHT / 2, 0);
    if (player) {
        strcpy(player->name, "Player");
        player->health = 100;
    }
    
    // Create some basic environment entities
    for (int i = 0; i < 10; i++) {
        float x = (rand() % CANVAS_WIDTH);
        float y = (rand() % CANVAS_HEIGHT);
        Entity* env = create_entity(x, y, 1 + (rand() % 3));
        if (env) {
            snprintf(env->name, sizeof(env->name), "Object_%d", i);
        }
    }
    
    g_initialized = true;
    printf("Game Engine v1.8 initialized successfully\n");
    printf("Initial entity count: %d\n", g_game_state->entity_count);
}

Entity* create_entity(float x, float y, int texture_id) {
    if (!g_game_state || g_game_state->entity_count >= MAX_ENTITIES) {
        return NULL;
    }
    
    Entity* entity = &g_game_state->entities[g_game_state->entity_count++];
    entity->x = x;
    entity->y = y;
    entity->z = 0;
    entity->velocity_x = 0;
    entity->velocity_y = 0;
    entity->rotation = 0;
    entity->texture_id = texture_id;
    entity->active = true;
    entity->health = 100;
    
    return entity;
}

void update_game_logic(double current_time) {
    if (g_game_state->paused) return;
    
    float delta_time = (float)(current_time - g_game_state->last_frame_time) / 1000.0f;
    g_game_state->last_frame_time = current_time;
    
    // Update FPS counter
    g_game_state->fps_counter++;
    g_game_state->fps_timer += delta_time;
    if (g_game_state->fps_timer >= 1.0) {
        printf("FPS: %d\n", g_game_state->fps_counter);
        g_game_state->fps_counter = 0;
        g_game_state->fps_timer = 0;
    }
    
    // Update all active entities
    for (int i = 0; i < g_game_state->entity_count; i++) {
        if (g_game_state->entities[i].active) {
            update_entity(&g_game_state->entities[i], delta_time);
        }
    }
    
    // Perform collision detection
    collision_detection();
    
    // Cleanup inactive entities
    cleanup_inactive_entities();
    
    // Simple AI for non-player entities
    for (int i = 1; i < g_game_state->entity_count; i++) {
        Entity* entity = &g_game_state->entities[i];
        if (entity->active && strcmp(entity->name, "Player") != 0) {
            // Simple movement pattern
            entity->velocity_x = sinf(current_time * 0.001f + i) * 50.0f;
            entity->velocity_y = cosf(current_time * 0.001f + i) * 50.0f;
        }
    }
}

void update_entity(Entity* entity, float delta_time) {
    // Apply velocity
    entity->x += entity->velocity_x * delta_time;
    entity->y += entity->velocity_y * delta_time;
    
    // Apply rotation
    entity->rotation += delta_time * 45.0f; // 45 degrees per second
    if (entity->rotation > 360.0f) entity->rotation -= 360.0f;
    
    // Boundary wrapping for classic arcade feel
    if (entity->x < 0) entity->x = CANVAS_WIDTH;
    if (entity->x > CANVAS_WIDTH) entity->x = 0;
    if (entity->y < 0) entity->y = CANVAS_HEIGHT;
    if (entity->y > CANVAS_HEIGHT) entity->y = 0;
    
    // Apply friction for classic physics
    entity->velocity_x *= 0.95f;
    entity->velocity_y *= 0.95f;
}

void collision_detection() {
    // Simple AABB collision detection for classic gameplay
    for (int i = 0; i < g_game_state->entity_count; i++) {
        if (!g_game_state->entities[i].active) continue;
        
        for (int j = i + 1; j < g_game_state->entity_count; j++) {
            if (!g_game_state->entities[j].active) continue;
            
            Entity* a = &g_game_state->entities[i];
            Entity* b = &g_game_state->entities[j];
            
            float dx = a->x - b->x;
            float dy = a->y - b->y;
            float distance = sqrtf(dx * dx + dy * dy);
            
            if (distance < 32.0f) { // Collision threshold
                // Simple collision response
                if (strcmp(a->name, "Player") == 0) {
                    g_game_state->score += 10;
                }
                
                // Bounce effect
                float bounce_force = 100.0f;
                a->velocity_x += (dx / distance) * bounce_force;
                a->velocity_y += (dy / distance) * bounce_force;
                b->velocity_x -= (dx / distance) * bounce_force;
                b->velocity_y -= (dy / distance) * bounce_force;
            }
        }
    }
}

void cleanup_inactive_entities() {
    // Compact entity array by removing inactive entities
    int write_index = 0;
    for (int read_index = 0; read_index < g_game_state->entity_count; read_index++) {
        if (g_game_state->entities[read_index].active) {
            if (write_index != read_index) {
                g_game_state->entities[write_index] = g_game_state->entities[read_index];
            }
            write_index++;
        }
    }
    g_game_state->entity_count = write_index;
}

void render_frame() {
    // This would typically interface with HTML5 Canvas
    // For now, we'll just update internal state and let JS handle rendering
    
    // Update camera to follow player (first entity)
    if (g_game_state->entity_count > 0) {
        Entity* player = &g_game_state->entities[0];
        g_game_state->camera_x = player->x;
        g_game_state->camera_y = player->y;
    }
}

void handle_input(int key_code, bool pressed) {
    if (g_game_state->entity_count == 0) return;
    
    Entity* player = &g_game_state->entities[0]; // Assume first entity is player
    float move_speed = 200.0f;
    
    if (pressed) {
        switch (key_code) {
            case 87: // W key
            case 38: // Up arrow
                player->velocity_y = -move_speed;
                break;
            case 83: // S key  
            case 40: // Down arrow
                player->velocity_y = move_speed;
                break;
            case 65: // A key
            case 37: // Left arrow
                player->velocity_x = -move_speed;
                break;
            case 68: // D key
            case 39: // Right arrow
                player->velocity_x = move_speed;
                break;
            case 32: // Space bar
                g_game_state->paused = !g_game_state->paused;
                break;
        }
    }
}

// Memory management functions for WASM GC
EMSCRIPTEN_KEEPALIVE
void wasm_cleanup() {
    if (g_game_state) {
        free(g_game_state);
        g_game_state = NULL;
    }
    g_initialized = false;
    printf("Game Engine v1.8 cleaned up\n");
}

// Export entity data for JavaScript rendering
EMSCRIPTEN_KEEPALIVE
float* wasm_get_entity_positions() {
    if (!g_game_state) return NULL;
    
    // Allocate array for entity positions (x, y, rotation for each entity)
    static float positions[MAX_ENTITIES * 3];
    
    for (int i = 0; i < g_game_state->entity_count; i++) {
        Entity* entity = &g_game_state->entities[i];
        positions[i * 3] = entity->x;
        positions[i * 3 + 1] = entity->y;
        positions[i * 3 + 2] = entity->rotation;
    }
    
    return positions;
}

EMSCRIPTEN_KEEPALIVE
float wasm_get_camera_x() {
    return g_game_state ? g_game_state->camera_x : 0;
}

EMSCRIPTEN_KEEPALIVE
float wasm_get_camera_y() {
    return g_game_state ? g_game_state->camera_y : 0;
}