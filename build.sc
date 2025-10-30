import mill._
import mill.scalalib._
import mill.scalalib.publish._
import mill.scalalib.Assembly._
import coursier.maven.MavenRepository

object v {
  val scala = "2.13.16"
  val chisel6 = "6.7.0"
  val chisel3 = "3.6.1"
  val chiselTest = "6.0.0"
  val sourcecode = "0.3.1"
  val mainargs = "0.5.0"
  val json4sJackson = "4.0.5"
  val scalaGraph = "1.13.5"
  val commonsLang3 = "3.12.0"
  val commonsText = "1.9"
  val reflections = "0.10.2"
  val scalaTest = "3.2.+"
  val spire = "0.18.0"
  val breeze = "2.1.0"
  val junit = "4.13"
  val scalaCheck = "1.14.3"
  val playJson = "2.9.2"
  val sonatypesSnapshots = Seq(
    MavenRepository("https://s01.oss.sonatype.org/content/repositories/snapshots"),
    MavenRepository("https://oss.sonatype.org/content/repositories/snapshots")
  )
}


trait CommonModule extends ScalaModule {
  def scalaVersion = v.scala

  override def repositoriesTask = T.task {
    super.repositoriesTask() ++ v.sonatypesSnapshots
  }

  override def scalacOptions = T {
    super.scalacOptions() ++ Seq(
      "-deprecation",
      "-unchecked",
      "-Ytasty-reader",
      "-Ymacro-annotations"
    )
  }

  override def ivyDeps = T {
    super.ivyDeps() ++ Agg(
      ivy"com.lihaoyi::sourcecode:${v.sourcecode}",
      ivy"org.scala-lang:scala-reflect:${v.scala}"
    )
  }
}

trait HasChisel extends CommonModule {
  def chiselVersion = T { v.chisel6 }

  override def ivyDeps = T {
    super.ivyDeps() ++ Agg(
      ivy"org.chipsalliance::chisel:${chiselVersion()}",
      ivy"org.apache.commons:commons-lang3:${v.commonsLang3}",
      ivy"org.apache.commons:commons-text:${v.commonsText}"
    )
  }

  override def scalacPluginIvyDeps = T {
    super.scalacPluginIvyDeps() ++ Agg(
      ivy"org.chipsalliance:::chisel-plugin:${chiselVersion()}"
    )
  }
}

// Chisel 3 settings for tapeout module
trait HasChisel3 extends CommonModule {
  override def scalaVersion = "2.13.10"

  override def ivyDeps = T {
    super.ivyDeps() ++ Agg(
      ivy"edu.berkeley.cs::chisel3:${v.chisel3}"
    )
  }

  override def scalacPluginIvyDeps = T {
    super.scalacPluginIvyDeps() ++ Agg(
      ivy"edu.berkeley.cs:::chisel3-plugin:${v.chisel3}"
    )
  }
}

trait HasScalaTest extends ScalaModule {
  override def ivyDeps = T {
    super.ivyDeps() ++ Agg(
      ivy"org.scalatest::scalatest:${v.scalaTest}"
    )
  }
}

object cde extends CommonModule {
  override def millSourcePath = os.pwd / "tools" / "cde"
  override def sources = T.sources {
    Seq(PathRef(millSourcePath / "cde" / "src" / "chipsalliance" / "rocketchip"))
  }
}

object midasTargetUtils extends HasChisel {
  override def millSourcePath = os.pwd / "sims" / "firesim" / "sim" / "midas" / "targetutils"
}

object hardfloat extends HasChisel with HasScalaTest {
  override def millSourcePath = os.pwd / "generators" / "hardfloat" / "hardfloat"
  override def sources = T.sources {
    Seq(PathRef(millSourcePath / "src" / "main" / "scala"))
  }
  override def moduleDeps = Seq(cde, midasTargetUtils)
}

object rocketMacros extends CommonModule with HasScalaTest {
  override def millSourcePath = os.pwd / "generators" / "rocket-chip" / "macros"
}

object diplomacy extends HasChisel with CommonModule {
  override def millSourcePath = os.pwd / "generators" / "diplomacy" / "diplomacy"
  override def sources = T.sources {
    Seq(PathRef(millSourcePath / "src"))
  }
  override def moduleDeps = Seq(cde)
}

object rocketchip extends HasChisel with HasScalaTest {
  override def millSourcePath = os.pwd / "generators" / "rocket-chip"
  override def moduleDeps = Seq(hardfloat, rocketMacros, diplomacy, cde)
  override def resources = T.sources {
    Seq(PathRef(millSourcePath / "src" / "main" / "resources"))
  }

  override def ivyDeps = T {
    super.ivyDeps() ++ Agg(
      ivy"com.lihaoyi::mainargs:${v.mainargs}",
      ivy"org.json4s::json4s-jackson:${v.json4sJackson}",
      ivy"org.scala-graph::graph-core:${v.scalaGraph}"
    )
  }
}

object boom extends HasChisel {
  override def millSourcePath = os.pwd / "generators" / "boom"
  override def moduleDeps = Seq(rocketchip)
  override def ivyDeps = T {
    super.ivyDeps() ++ rocketchip.ivyDeps()
  }
}

object testchipip extends HasChisel {
  override def millSourcePath = os.pwd / "generators" / "testchipip"
  override def moduleDeps = Seq(rocketchip, rocketchipBlocks)
  override def resources = T.sources {
    Seq(PathRef(millSourcePath / "src" / "main" / "resources"))
  }
  override def ivyDeps = T {
    super.ivyDeps() ++ rocketchip.ivyDeps()
  }
}

object rocketchipBlocks extends HasChisel {
  override def millSourcePath = os.pwd / "generators" / "rocket-chip-blocks"
  override def moduleDeps = Seq(rocketchip)
  override def ivyDeps = T {
    super.ivyDeps() ++ rocketchip.ivyDeps()
  }
}

object rocketchipInclusiveCache extends HasChisel {
  override def millSourcePath = os.pwd / "generators" / "rocket-chip-inclusive-cache"
  override def sources = T.sources {
    Seq(PathRef(millSourcePath / "design" / "craft"))
  }
  override def moduleDeps = Seq(rocketchip)
  override def ivyDeps = T {
    super.ivyDeps() ++ rocketchip.ivyDeps()
  }
}

object icenet extends HasChisel {
  override def millSourcePath = os.pwd / "generators" / "icenet"
  override def moduleDeps = Seq(rocketchip)
  override def ivyDeps = T {
    super.ivyDeps() ++ rocketchip.ivyDeps()
  }
}

object tracegen extends HasChisel {
  override def millSourcePath = os.pwd / "generators" / "tracegen"
  override def moduleDeps = Seq(testchipip, rocketchip, rocketchipInclusiveCache, boom)
  override def ivyDeps = T {
    super.ivyDeps() ++ rocketchip.ivyDeps()
  }
}

object constellation extends HasChisel {
  override def millSourcePath = os.pwd / "generators" / "constellation"
  // Exclude test sources for now (they have chiseltest dependencies)
  override def sources = T.sources {
    Seq(PathRef(millSourcePath / "src" / "main" / "scala"))
  }
  override def moduleDeps = Seq(rocketchip)
  override def ivyDeps = T {
    super.ivyDeps() ++ rocketchip.ivyDeps()
  }
}

object barf extends HasChisel {
  override def millSourcePath = os.pwd / "generators" / "bar-fetchers"
  override def moduleDeps = Seq(rocketchip)
  override def ivyDeps = T {
    super.ivyDeps() ++ rocketchip.ivyDeps()
  }
}

object shuttle extends HasChisel {
  override def millSourcePath = os.pwd / "generators" / "shuttle"
  override def moduleDeps = Seq(rocketchip)
  override def ivyDeps = T {
    super.ivyDeps() ++ rocketchip.ivyDeps()
  }
}

object saturn extends CommonModule {
  override def millSourcePath = os.pwd / "generators" / "saturn"
  override def moduleDeps = Seq(rocketchip, shuttle)
  override def ivyDeps = T {
    super.ivyDeps() ++ rocketchip.ivyDeps()
  }
}

object gemmini extends HasChisel {
  override def millSourcePath = os.pwd / "generators" / "gemmini"
  override def sources = T.sources {
    Seq(PathRef(millSourcePath / "src" / "main" / "scala"))
  }
  override def moduleDeps = Seq(rocketchip)
  override def ivyDeps = T {
    super.ivyDeps() ++ rocketchip.ivyDeps()
  }
}

object firrtl2 extends CommonModule {
  override def millSourcePath = os.pwd / "tools" / "firrtl2"
  override def sources = T.sources {
    val mainScala = PathRef(millSourcePath / "src" / "main" / "scala")
    val generatedSources = PathRef(millSourcePath / "src" / "target" / "scala-2.13" / "src_managed" / "main")
    Seq(mainScala, generatedSources)
  }

  override def scalacOptions = T {
    super.scalacOptions() ++ Seq(
      "-language:reflectiveCalls",
      "-language:existentials",
      "-language:implicitConversions"
    )
  }

  override def ivyDeps = T {
    super.ivyDeps() ++ Agg(
      ivy"org.scalatest::scalatest:3.2.14",
      ivy"com.github.scopt::scopt:4.1.0",
      ivy"org.json4s::json4s-native:4.1.0-M4",
      ivy"org.apache.commons:commons-text:1.10.0",
      ivy"com.lihaoyi::os-lib:0.8.1",
      ivy"org.scala-lang.modules::scala-parallel-collections:1.0.4",
      ivy"org.antlr:antlr4-runtime:4.9.3"
    )
  }
}

object firrtl2Bridge extends HasChisel {
  override def millSourcePath = os.pwd / "tools" / "firrtl2" / "bridge"
  override def sources = T.sources {
    Seq(PathRef(millSourcePath / "src" / "main" / "scala"))
  }
  override def moduleDeps = Seq(firrtl2)
}

object firesimLib extends HasChisel with HasScalaTest {
  override def millSourcePath = os.pwd / "sims" / "firesim" / "sim" / "firesim-lib"
  override def moduleDeps = Seq(midasTargetUtils)
}

object firechipBridgeInterfaces extends HasChisel {
  override def millSourcePath = os.pwd / "generators" / "firechip" / "bridgeinterfaces"
}

object firechipBridgeStubs extends HasChisel with HasScalaTest {
  override def millSourcePath = os.pwd / "generators" / "firechip" / "bridgestubs"
  override def moduleDeps = Seq(chipyard, firesimLib, firechipBridgeInterfaces)
}

object firechip extends HasChisel with HasScalaTest {
  override def millSourcePath = os.pwd / "generators" / "firechip" / "chip"
  override def moduleDeps = Seq(chipyard, firesimLib, firechipBridgeStubs, firechipBridgeInterfaces)
}

object fixedpoint extends HasChisel {
  override def millSourcePath = os.pwd / "tools" / "fixedpoint"
  override def sources = T.sources {
    Seq(PathRef(millSourcePath / "src" / "main" / "scala"))
  }
}

object dsptools extends HasChisel with HasScalaTest {
  override def millSourcePath = os.pwd / "tools" / "dsptools"
  override def sources = T.sources {
    Seq(PathRef(millSourcePath / "src" / "main" / "scala"))
  }
  override def moduleDeps = Seq(fixedpoint)

  override def ivyDeps = T {
    super.ivyDeps() ++ Agg(
      ivy"edu.berkeley.cs::chiseltest:${v.chiselTest}",
      ivy"org.typelevel::spire:${v.spire}",
      ivy"org.scalanlp::breeze:${v.breeze}",
      ivy"junit:junit:${v.junit}",
      ivy"org.scalacheck::scalacheck:${v.scalaCheck}"
    )
  }
}

object rocketDspUtils extends HasChisel {
  override def millSourcePath = os.pwd / "tools" / "rocket-dsp-utils"
  override def sources = T.sources {
    Seq(PathRef(millSourcePath / "src" / "main" / "scala"))
  }
  override def moduleDeps = Seq(rocketchip, cde, dsptools)
  override def ivyDeps = T {
    super.ivyDeps() ++ rocketchip.ivyDeps()
  }
}

object fpgaShells extends CommonModule {
  override def millSourcePath = os.pwd / "fpga" / "fpga-shells"
  override def moduleDeps = Seq(rocketchip, rocketchipBlocks)
  override def ivyDeps = T {
    super.ivyDeps() ++ rocketchip.ivyDeps()
  }
}

object chipyardFpga extends CommonModule {
  override def millSourcePath = os.pwd / "fpga"
  override def moduleDeps = Seq(chipyard, fpgaShells)
}

object tapeout extends HasChisel3 {
  override def millSourcePath = os.pwd / "tools" / "tapeout"

  override def ivyDeps = T {
    super.ivyDeps() ++ Agg(
      ivy"com.typesafe.play::play-json:${v.playJson}"
    )
  }
}

object chipyard extends HasChisel {
  override def millSourcePath = os.pwd / "generators" / "chipyard"

  override def moduleDeps = Seq(
    testchipip, rocketchip, boom, rocketchipBlocks, rocketchipInclusiveCache,
    icenet, tracegen, constellation, barf, shuttle, firrtl2Bridge, dsptools, rocketDspUtils
  )
  
  override def sources = T.sources {
    val baseDir = millSourcePath / "src" / "main" / "scala"
    val stageDir = os.pwd / "tools" / "stage" / "src" / "main" / "scala"

    val excludeDirs = Seq(
      baseDir / "example" / "dsptools",  // Exclude dsptools examples
      baseDir / "config" / "MMIOAcceleratorConfigs.scala",
      baseDir / "config" / "TutorialConfigs.scala",
      baseDir / "upf"
    ).filter(p => os.exists(p))

    val baseSources = if (os.exists(baseDir)) {
      os.walk(baseDir)
        .filter(_.ext == "scala")
        .filterNot(f => excludeDirs.exists(d => f.startsWith(d)))
    } else Seq()

    val stageSources = if (os.exists(stageDir)) {
      os.walk(stageDir).filter(_.ext == "scala")
    } else Seq()

    val optional = discoverOptionalSources()

    Seq(PathRef(baseDir)) ++ optional ++ Seq(PathRef(stageDir))
  }

  override def resources = T.sources {
    Seq(PathRef(millSourcePath / "src" / "main" / "resources"))
  }

  override def unmanagedClasspath = T {
    super.unmanagedClasspath() ++ Agg(
      PathRef(os.pwd / "lib")
    ).filter(p => os.exists(p.path))
  }

  override def ivyDeps = T {
    super.ivyDeps() ++ rocketchip.ivyDeps() ++ Agg(
      ivy"org.reflections:reflections:${v.reflections}"
    )
  }

  // Assembly settings for JAR generation - handle merge conflicts
  override def assemblyRules = super.assemblyRules

  override def mainClass = T {
    Some("chipyard.Generator")
  }

  def discoverOptionalModules() = {
    val optionalMap = Map(
      "saturn" -> (() => saturn),
      "gemmini" -> (() => gemmini)
    )

    optionalMap.filter { case (dir, _) =>
      os.exists(os.pwd / "generators" / dir / ".git")
    }.values.map(_()).toSeq
  }

  def discoverOptionalSources() = {
    val optionalDirs = Seq("saturn", "gemmini")
    optionalDirs.filter { dir =>
      val chipyardDir = os.pwd / "generators" / dir / "chipyard"
      os.exists(chipyardDir)
    }.map(dir => PathRef(os.pwd / "generators" / dir / "chipyard"))
  }
}

object chipyardRoot extends Module {
  def generateClasspath = T {
    chipyard.assembly()
  }
}
