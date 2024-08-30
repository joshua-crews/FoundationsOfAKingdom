extern crate directories;
use directories::UserDirs;
use std::fs;

use noise::utils::{NoiseImage, NoiseMap};
use noise::{core::worley::ReturnType, utils::*, *};

use crate::config_parser::*;

#[allow(non_snake_case)]
pub fn generate_texture(map_config: &MapConfig, engine_config: &EngineConfig) -> NoiseMap {
    fn baseContinentDef(map_config: &MapConfig) -> impl NoiseFn<f64, 3> {
        let baseContinentDef_fb0 = Fbm::<Perlin>::new(map_config.seed)
            .set_frequency(map_config.continent_frequency)
            .set_persistence(0.5)
            .set_lacunarity(map_config.continent_lacunarity)
            .set_octaves(5);
        let baseContinentDef_cu = Curve::new(baseContinentDef_fb0)
            .add_control_point(
                -2.0000 + map_config.sea_level,
                -1.625 + map_config.sea_level,
            )
            .add_control_point(
                -1.0000 + map_config.sea_level,
                -1.375 + map_config.sea_level,
            )
            .add_control_point(0.0000 + map_config.sea_level, -0.375 + map_config.sea_level)
            .add_control_point(0.0625 + map_config.sea_level, 0.125 + map_config.sea_level)
            .add_control_point(0.1250 + map_config.sea_level, 0.250 + map_config.sea_level)
            .add_control_point(0.2500 + map_config.sea_level, 1.000 + map_config.sea_level)
            .add_control_point(0.5000 + map_config.sea_level, 0.250 + map_config.sea_level)
            .add_control_point(0.7500 + map_config.sea_level, 0.250 + map_config.sea_level)
            .add_control_point(1.0000 + map_config.sea_level, 0.500 + map_config.sea_level)
            .add_control_point(2.0000 + map_config.sea_level, 0.500 + map_config.sea_level);

        let baseContinentDef_fb1 = Fbm::<Perlin>::new(map_config.seed + 1)
            .set_frequency(map_config.continent_frequency * 4.34375)
            .set_persistence(0.5)
            .set_lacunarity(map_config.continent_lacunarity)
            .set_octaves(11);

        let baseContinentDef_sb = ScaleBias::new(baseContinentDef_fb1)
            .set_scale(0.375)
            .set_bias(0.625);

        let baseContinentDef_mi = Min::new(baseContinentDef_sb, baseContinentDef_cu);

        let baseContinentDef_cl = Clamp::new(baseContinentDef_mi).set_bounds(-1.0, 1.0);

        let baseContinentDef = Cache::new(baseContinentDef_cl);

        baseContinentDef
    }

    let continentDef_tu0 = Turbulence::<_, Perlin>::new(baseContinentDef(&map_config))
        .set_seed(map_config.seed + 10)
        .set_frequency(map_config.continent_frequency * 15.25)
        .set_power(map_config.continent_frequency / 113.75)
        .set_roughness(13);

    let continentDef_tu1 = Turbulence::<_, Perlin>::new(continentDef_tu0)
        .set_seed(map_config.seed + 11)
        .set_frequency(map_config.continent_frequency * 47.25)
        .set_power(map_config.continent_frequency / 433.75)
        .set_roughness(12);

    let continentDef_tu2 = Turbulence::<_, Perlin>::new(continentDef_tu1)
        .set_seed(map_config.seed + 12)
        .set_frequency(map_config.continent_frequency * 95.25)
        .set_power(map_config.continent_frequency / 1019.75)
        .set_roughness(11);

    let continentDef_se = Select::new(
        baseContinentDef(&map_config),
        continentDef_tu2,
        baseContinentDef(&map_config),
    )
    .set_bounds(
        map_config.sea_level - 0.0375,
        map_config.sea_level + 1000.0375,
    )
    .set_falloff(0.0625);

    let continentDef = Cache::new(continentDef_se);

    let terrainTypeDef_tu = Turbulence::<_, Perlin>::new(&continentDef)
        .set_seed(map_config.seed + 20)
        .set_frequency(map_config.continent_frequency * 18.125)
        .set_power(map_config.continent_frequency / 20.59375 * map_config.terrain_offset)
        .set_roughness(3);

    let terrainTypeDef_te = Terrace::new(terrainTypeDef_tu)
        .add_control_point(-1.00)
        .add_control_point(map_config.shelf_level + map_config.sea_level / 2.0)
        .add_control_point(1.00);

    let terrainTypeDef = Cache::new(terrainTypeDef_te);

    let mountainBaseDef_rm0 = RidgedMulti::<Perlin>::new(map_config.seed + 30)
        .set_frequency(1723.0)
        .set_lacunarity(map_config.mountain_lacunarity)
        .set_octaves(4);

    let mountainBaseDef_sb0 = ScaleBias::new(mountainBaseDef_rm0)
        .set_scale(0.5)
        .set_bias(0.375);

    let mountainBaseDef_rm1 = RidgedMulti::<Perlin>::new(map_config.seed + 31)
        .set_frequency(367.0)
        .set_lacunarity(map_config.mountain_lacunarity)
        .set_octaves(1);

    let mountainBaseDef_sb1 = ScaleBias::new(mountainBaseDef_rm1)
        .set_scale(-2.0)
        .set_bias(-0.5);

    let mountainBaseDef_co = Constant::new(-1.0);

    let mountainBaseDef_bl = Blend::new(
        &mountainBaseDef_co,
        &mountainBaseDef_sb0,
        &mountainBaseDef_sb1,
    );

    let mountainBaseDef_tu0 = Turbulence::<_, Perlin>::new(mountainBaseDef_bl)
        .set_seed(map_config.seed + 32)
        .set_frequency(1337.0)
        .set_power(1.0 / 6730.0 * map_config.mountains_twist)
        .set_roughness(4);

    let mountainBaseDef_tu1 = Turbulence::<_, Perlin>::new(mountainBaseDef_tu0)
        .set_seed(map_config.seed + 33)
        .set_frequency(21221.0)
        .set_power(1.0 / 120157.0 * map_config.mountains_twist)
        .set_roughness(6);

    let mountainBaseDef = Cache::new(mountainBaseDef_tu1);

    let mountainousHigh_rm0 = RidgedMulti::<Perlin>::new(map_config.seed + 40)
        .set_frequency(2371.0)
        .set_lacunarity(map_config.mountain_lacunarity)
        .set_octaves(3);

    let mountainousHigh_rm1 = RidgedMulti::<Perlin>::new(map_config.seed + 41)
        .set_frequency(2341.0)
        .set_lacunarity(map_config.mountain_lacunarity)
        .set_octaves(3);

    let mountainousHigh_ma = Max::new(mountainousHigh_rm0, mountainousHigh_rm1);

    let mountainousHigh_tu = Turbulence::<_, Perlin>::new(mountainousHigh_ma)
        .set_seed(map_config.seed + 42)
        .set_frequency(31511.0)
        .set_power(1.0 / 180371.0 * map_config.mountains_twist)
        .set_roughness(4);

    let mountainousHigh = Cache::new(mountainousHigh_tu);

    let mountainousLow_rm0 = RidgedMulti::<Perlin>::new(map_config.seed + 50)
        .set_frequency(1381.0)
        .set_lacunarity(map_config.mountain_lacunarity)
        .set_octaves(8);

    let mountainousLow_rm1 = RidgedMulti::<Perlin>::new(map_config.seed + 51)
        .set_frequency(1427.0)
        .set_lacunarity(map_config.mountain_lacunarity)
        .set_octaves(8);

    let mountainousLow_mu = Multiply::new(mountainousLow_rm0, mountainousLow_rm1);

    let mountainousLow = Cache::new(mountainousLow_mu);

    let mountainousTerrain_sb0 = ScaleBias::new(mountainousLow)
        .set_scale(0.03125)
        .set_bias(-0.96875);

    let mountainousTerrain_sb1 = ScaleBias::new(mountainousHigh)
        .set_scale(0.25)
        .set_bias(0.25);

    let mountainousTerrain_ad = Add::new(mountainousTerrain_sb1, &mountainBaseDef);

    let mountainousTerrain_se = Select::new(
        mountainousTerrain_sb0,
        mountainousTerrain_ad,
        &mountainBaseDef,
    )
    .set_bounds(-0.5, 999.5)
    .set_falloff(0.5);

    let mountainousTerrain_sb2 = ScaleBias::new(mountainousTerrain_se)
        .set_scale(0.8)
        .set_bias(0.0);

    let mountainousTerrain_ex =
        Exponent::new(mountainousTerrain_sb2).set_exponent(map_config.mountain_glaciation);

    let mountainousTerrain = Cache::new(mountainousTerrain_ex);

    let hillyTerrain_bi = Billow::<Perlin>::new(map_config.seed + 60)
        .set_frequency(1663.0)
        .set_persistence(0.5)
        .set_lacunarity(map_config.hills_lacunarity)
        .set_octaves(6);

    let hillyTerrain_sb0 = ScaleBias::new(hillyTerrain_bi).set_scale(0.5).set_bias(0.5);

    let hillyTerrain_rm = RidgedMulti::<Perlin>::new(map_config.seed + 61)
        .set_frequency(367.5)
        .set_lacunarity(map_config.hills_lacunarity)
        .set_octaves(1);

    let hillyTerrain_sb1 = ScaleBias::new(hillyTerrain_rm)
        .set_scale(-2.0)
        .set_bias(-1.0);

    let hillyTerrain_co = Constant::new(-1.0);

    let hillyTerrain_bl = Blend::new(hillyTerrain_co, hillyTerrain_sb1, hillyTerrain_sb0);

    let hillyTerrain_sb2 = ScaleBias::new(hillyTerrain_bl)
        .set_scale(0.75)
        .set_bias(-0.25);

    let hillyTerrain_ex = Exponent::new(hillyTerrain_sb2).set_exponent(1.375);

    let hillyTerrain_tu0 = Turbulence::<_, Perlin>::new(hillyTerrain_ex)
        .set_seed(map_config.seed + 62)
        .set_frequency(1531.0)
        .set_power(1.0 / 16921.0 * map_config.hills_twist)
        .set_roughness(4);

    let hillyTerrain_tu1 = Turbulence::<_, Perlin>::new(hillyTerrain_tu0)
        .set_seed(map_config.seed + 63)
        .set_frequency(21617.0)
        .set_power(1.0 / 117529.0 * map_config.hills_twist)
        .set_roughness(6);

    let hillyTerrain = Cache::new(hillyTerrain_tu1);

    let plainsTerrain_bi0 = Billow::<Perlin>::new(map_config.seed + 70)
        .set_frequency(1097.5)
        .set_persistence(0.5)
        .set_lacunarity(map_config.plains_lacunarity)
        .set_octaves(8);

    let plainsTerrain_sb0 = ScaleBias::new(plainsTerrain_bi0)
        .set_scale(0.5)
        .set_bias(0.5);

    let plainsTerrain_bi1 = Billow::<Perlin>::new(map_config.seed + 71)
        .set_frequency(1097.5)
        .set_persistence(0.5)
        .set_lacunarity(map_config.plains_lacunarity)
        .set_octaves(8);

    let plainsTerrain_sb1 = ScaleBias::new(plainsTerrain_bi1)
        .set_scale(0.5)
        .set_bias(0.5);

    let plainsTerrain_mu = Multiply::new(plainsTerrain_sb0, plainsTerrain_sb1);

    let plainsTerrain_sb2 = ScaleBias::new(plainsTerrain_mu)
        .set_scale(2.0)
        .set_bias(-1.0);

    let plainsTerrain = Cache::new(plainsTerrain_sb2);

    let badlandsSand_rm = RidgedMulti::<Perlin>::new(map_config.seed + 80)
        .set_frequency(6163.5)
        .set_lacunarity(map_config.badlands_lacunarity)
        .set_octaves(1);

    let badlandsSand_sb0 = ScaleBias::new(badlandsSand_rm)
        .set_scale(0.875)
        .set_bias(0.0);

    let badlandsSand_wo = Worley::new(map_config.seed + 81)
        .set_frequency(16183.25)
        .set_return_type(ReturnType::Distance);

    let badlandsSand_sb1 = ScaleBias::new(badlandsSand_wo)
        .set_scale(0.25)
        .set_bias(0.25);

    let badlandsSand_ad = Add::new(badlandsSand_sb0, badlandsSand_sb1);

    let badlandsSand = Cache::new(badlandsSand_ad);

    let badlandsCliffs_fb = Fbm::<Perlin>::new(map_config.seed + 90)
        .set_frequency(map_config.continent_frequency * 839.0)
        .set_persistence(0.5)
        .set_lacunarity(map_config.badlands_lacunarity)
        .set_octaves(6);

    let badlandsCliffs_cu = Curve::new(badlandsCliffs_fb)
        .add_control_point(-2.000, -2.000)
        .add_control_point(-1.000, -1.000)
        .add_control_point(-0.000, -0.750)
        .add_control_point(0.500, -0.250)
        .add_control_point(0.625, 0.875)
        .add_control_point(0.750, 1.000)
        .add_control_point(2.000, 1.250);

    let badlandsCliffs_cl = Clamp::new(badlandsCliffs_cu).set_bounds(-999.125, 0.875);

    let badlandsCliffs_te = Terrace::new(badlandsCliffs_cl)
        .add_control_point(-1.000)
        .add_control_point(-0.875)
        .add_control_point(-0.750)
        .add_control_point(-0.500)
        .add_control_point(0.000)
        .add_control_point(1.000);

    let badlandsCliffs_tu0 = Turbulence::<_, Perlin>::new(badlandsCliffs_te)
        .set_seed(map_config.seed + 91)
        .set_frequency(16111.0)
        .set_power(1.0 / 141539.0 * map_config.badlands_twist)
        .set_roughness(3);

    let badlandsCliffs_tu1 = Turbulence::<_, Perlin>::new(badlandsCliffs_tu0)
        .set_seed(map_config.seed + 92)
        .set_frequency(36107.0)
        .set_power(1.0 / 211543.0 * map_config.badlands_twist)
        .set_roughness(3);

    let badlandsCliffs = Cache::new(badlandsCliffs_tu1);

    let badlandsTerrain_sb = ScaleBias::new(badlandsSand).set_scale(0.25).set_bias(-0.75);

    let badlandsTerrain_ma = Max::new(badlandsCliffs, badlandsTerrain_sb);

    let badlandsTerrain = Cache::new(badlandsTerrain_ma);

    let riverPositions_rm0 = RidgedMulti::<Perlin>::new(map_config.seed + 100)
        .set_frequency(18.75)
        .set_lacunarity(map_config.continent_lacunarity)
        .set_octaves(1);

    let riverPositions_cu0 = Curve::new(riverPositions_rm0)
        .add_control_point(-2.000, 2.000)
        .add_control_point(-1.000, 1.000)
        .add_control_point(-0.125, 0.875)
        .add_control_point(0.000, -1.000)
        .add_control_point(1.000, -1.500)
        .add_control_point(2.000, -2.000);

    let riverPositions_rm1 = RidgedMulti::<Perlin>::new(map_config.seed + 101)
        .set_frequency(43.25)
        .set_lacunarity(map_config.continent_lacunarity)
        .set_octaves(1);

    let riverPositions_cu1 = Curve::new(riverPositions_rm1)
        .add_control_point(-2.000, 2.0000)
        .add_control_point(-1.000, 1.5000)
        .add_control_point(-0.125, 1.4375)
        .add_control_point(0.000, 0.5000)
        .add_control_point(1.000, 0.2500)
        .add_control_point(2.000, 0.0000);

    let riverPositions_mi = Min::new(riverPositions_cu0, riverPositions_cu1);

    let riverPositions_tu = Turbulence::<_, Perlin>::new(riverPositions_mi)
        .set_seed(map_config.seed + 102)
        .set_frequency(9.25)
        .set_power(1.0 / 57.75)
        .set_roughness(6);

    let riverPositions = Cache::new(riverPositions_tu);

    let scaledMountainousTerrain_sb0 = ScaleBias::new(mountainousTerrain)
        .set_scale(0.125)
        .set_bias(0.125);

    let scaledMountainousTerrain_fb = Fbm::<Perlin>::new(map_config.seed + 110)
        .set_frequency(14.5)
        .set_persistence(0.5)
        .set_lacunarity(map_config.mountain_lacunarity)
        .set_octaves(6);

    let scaledMountainousTerrain_ex = Exponent::new(scaledMountainousTerrain_fb).set_exponent(1.25);

    let scaledMountainousTerrain_sb1 = ScaleBias::new(scaledMountainousTerrain_ex)
        .set_scale(0.25)
        .set_bias(1.0);

    let scaledMountainousTerrain_mu =
        Multiply::new(scaledMountainousTerrain_sb0, scaledMountainousTerrain_sb1);

    let scaledMountainousTerrain = Cache::new(scaledMountainousTerrain_mu);

    let scaledHillyTerrain_sb0 = ScaleBias::new(hillyTerrain)
        .set_scale(0.0625)
        .set_bias(0.0625);

    let scaledHillyTerrain_fb = Fbm::<Perlin>::new(map_config.seed + 120)
        .set_frequency(13.5)
        .set_persistence(0.5)
        .set_lacunarity(map_config.hills_lacunarity)
        .set_octaves(6);

    let scaledHillyTerrain_ex = Exponent::new(scaledHillyTerrain_fb).set_exponent(1.25);

    let scaledHillyTerrain_sb1 = ScaleBias::new(scaledHillyTerrain_ex)
        .set_scale(0.5)
        .set_bias(1.5);

    let scaledHillyTerrain_mu = Multiply::new(scaledHillyTerrain_sb0, scaledHillyTerrain_sb1);

    let scaledHillyTerrain = Cache::new(scaledHillyTerrain_mu);

    let scaledPlainsTerrain_sb0 = ScaleBias::new(plainsTerrain)
        .set_scale(0.00390625)
        .set_bias(0.0078125);

    let scaledPlainsTerrain = Cache::new(scaledPlainsTerrain_sb0);

    let scaledBadlandsTerrain_sb = ScaleBias::new(badlandsTerrain)
        .set_scale(0.0625)
        .set_bias(0.0625);

    let scaledBadlandsTerrain = Cache::new(scaledBadlandsTerrain_sb);

    let continentalShelf_te = Terrace::new(&continentDef)
        .add_control_point(-1.0)
        .add_control_point(-0.75)
        .add_control_point(map_config.shelf_level)
        .add_control_point(1.0);

    let continentalShelf_cl =
        Clamp::new(continentalShelf_te).set_bounds(-0.75, map_config.sea_level);

    let continentalShelf_rm = RidgedMulti::<Perlin>::new(map_config.seed + 130)
        .set_frequency(map_config.continent_frequency * 4.375)
        .set_lacunarity(map_config.continent_lacunarity)
        .set_octaves(16);

    let continentalShelf_sb = ScaleBias::new(continentalShelf_rm)
        .set_scale(-0.125)
        .set_bias(-0.125);

    let continentalShelf_ad = Add::new(continentalShelf_sb, continentalShelf_cl);

    let continentalShelf = Cache::new(continentalShelf_ad);

    let baseContinentElev_sb = ScaleBias::new(&continentDef)
        .set_scale(map_config.continent_height_scale)
        .set_bias(0.0);

    let baseContinentElev_se = Select::new(baseContinentElev_sb, continentalShelf, &continentDef)
        .set_bounds(map_config.shelf_level - 1000.0, map_config.shelf_level)
        .set_falloff(0.03125);

    let baseContinentElev = Cache::new(baseContinentElev_se);

    let continentsWithPlains_ad = Add::new(&baseContinentElev, scaledPlainsTerrain);

    let continentsWithPlains = Cache::new(continentsWithPlains_ad);

    let continentsWithHills_ad = Add::new(&baseContinentElev, scaledHillyTerrain);

    let continentsWithHills_se = Select::new(
        &continentsWithPlains,
        &continentsWithHills_ad,
        &terrainTypeDef,
    )
    .set_bounds(
        1.0 - map_config.hills_amount,
        1001.0 - map_config.hills_amount,
    )
    .set_falloff(0.25);

    let continentsWithHills = Cache::new(continentsWithHills_se);

    let continentsWithMountains_ad0 = Add::new(&baseContinentElev, scaledMountainousTerrain);

    let continentsWithMountains_cu = Curve::new(&continentDef)
        .add_control_point(-1.0, -0.0625)
        .add_control_point(0.0, 0.0000)
        .add_control_point(1.0 - map_config.mountains_amount, 0.0625)
        .add_control_point(1.0, 0.2500);

    let continentsWithMountains_ad1 =
        Add::new(continentsWithMountains_ad0, continentsWithMountains_cu);

    let continentsWithMountains_se = Select::new(
        continentsWithHills,
        continentsWithMountains_ad1,
        &terrainTypeDef,
    )
    .set_bounds(
        1.0 - map_config.mountains_amount,
        1001.0 - map_config.mountains_amount,
    )
    .set_falloff(0.25);

    let continentsWithMountains = Cache::new(continentsWithMountains_se);

    let continentsWithBadlands_bm = Fbm::<Perlin>::new(map_config.seed + 140)
        .set_frequency(16.5)
        .set_persistence(0.5)
        .set_lacunarity(map_config.continent_lacunarity)
        .set_octaves(2);

    let continentsWithBadlands_ad = Add::new(&baseContinentElev, scaledBadlandsTerrain);

    let continentsWithBadlands_se = Select::new(
        &continentsWithMountains,
        &continentsWithBadlands_ad,
        &continentsWithBadlands_bm,
    )
    .set_bounds(
        1.0 - map_config.badlands_amount,
        1001.0 - map_config.badlands_amount,
    )
    .set_falloff(0.25);

    let continentsWithBadlands_ma = Max::new(&continentsWithMountains, continentsWithBadlands_se);

    let continentsWithBadlands = Cache::new(continentsWithBadlands_ma);

    let continentsWithRivers_sb = ScaleBias::new(riverPositions)
        .set_scale(map_config.river_depth / 2.0)
        .set_bias(-map_config.river_depth / 2.0);

    let continentsWithRivers_ad = Add::new(&continentsWithBadlands, continentsWithRivers_sb);

    let continentsWithRivers_se = Select::new(
        &continentsWithBadlands,
        continentsWithRivers_ad,
        &continentsWithBadlands,
    )
    .set_bounds(
        map_config.sea_level,
        map_config.continent_height_scale + map_config.sea_level,
    )
    .set_falloff(map_config.continent_height_scale - map_config.sea_level);

    let continentsWithRivers = Cache::new(continentsWithRivers_se);

    let unscaledFinalPlanet = Cache::new(continentsWithRivers);

    let noise_map = PlaneMapBuilder::new(&unscaledFinalPlanet)
        .set_size(engine_config.world_size, engine_config.world_size)
        .set_x_bounds(-2.0, 2.0)
        .set_y_bounds(-2.0, 2.0)
        .build();

    write_image_to_file(
        &ImageRenderer::new()
            .set_gradient(ColorGradient::new().build_terrain_gradient())
            .render(&noise_map),
        "fbm_perlin.png",
    );

    return noise_map;
}

fn write_image_to_file(_image: &NoiseImage, filename: &str) {
    if let Some(user_dirs) = UserDirs::new() {
        if let Some(documents_dir) = user_dirs.document_dir() {
            let target = documents_dir
                .join("My Games")
                .join("Foundations of a Kingdom")
                .join("Saves")
                .join(filename);

            if let Some(parent_dir) = target.parent() {
                fs::create_dir_all(parent_dir).expect("Failed to create directories.");
            }

            //TODO Write image to file
            _image.write_to_file(&target);
        } else {
            println!("Documents directory not found.");
        }
    } else {
        println!("User directories not found.");
    }
}
