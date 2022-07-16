using Microsoft.OpenApi.Models;
using System.Reflection;

namespace api
{
    public class Program
    {
        public static void Main(string[] args)
        {
            WebApplicationBuilder? builder = WebApplication.CreateBuilder(args);

            // Add services to the container.
            builder.Services.AddControllers();

            // Learn more about configuring Swagger/OpenAPI at https://aka.ms/aspnetcore/swashbuckle
            builder.Services.AddEndpointsApiExplorer();
            builder.Services.AddSwaggerGen(options => {
                options.SwaggerDoc("v1", new OpenApiInfo {
                    Version = "v1",
                    Title = "Deps graph",
                    Description = "Get graph of your dependencies!",
                    TermsOfService = new Uri("https://github.com/TDiblik/deps-graph/blob/master/LICENSE"),
                    Contact = new OpenApiContact {
                        Name = "Tomáš Diblík",
                        Email = "dibla.tomas@post.cz",
                        Url = new Uri("https://tomasdiblik.cz/")
                    },
                    License = new OpenApiLicense {
                        Name = "License",
                        Url = new Uri("https://github.com/TDiblik/deps-graph/blob/master/LICENSE")
                    }
                });

                string? xmlFilename = $"{Assembly.GetExecutingAssembly().GetName().Name}.xml";
                options.IncludeXmlComments(Path.Combine(AppContext.BaseDirectory, xmlFilename));
            });

            WebApplication? app = builder.Build();

            // Since this is open source, it's ok to use swagger in production.
            app.UseSwagger();
            app.UseSwaggerUI();

            app.UseHttpsRedirection();

            app.UseAuthorization();

            app.MapControllers();

            app.Run();
        }
    }
}